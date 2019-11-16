use crate::{PixelBufferFormatType, PixelBufferFormatSupported, PixelBufferCreationError};
use winapi::{
    shared::windef::{HBITMAP, HWND},
    um::{wingdi::{self, BITMAP, BITMAPINFOHEADER}, winuser},
};
use raw_window_handle::{RawWindowHandle, windows::WindowsHandle};
use std::{convert::TryInto, ptr, io};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

pub struct PixelBuffer {
    handle: HBITMAP,
    bitmap: BITMAP,
    len: usize,
    hwnd: HWND,
}

unsafe impl Send for PixelBuffer {}

fn px_cast(u: u32) -> i32 {
    u.try_into().expect("Pixel value too large; must be less than 2,147,483,647")
}

impl PixelBufferFormatSupported for crate::BGRA {}
impl PixelBufferFormatSupported for crate::BGR {}
pub type NativeFormat = crate::BGRA;

fn hwnd(handle: RawWindowHandle) -> HWND {
    match handle {
        RawWindowHandle::Windows(WindowsHandle{hwnd, ..}) => hwnd as _,
        _ => panic!("Unsupported window handle type"),
    }
}

impl PixelBuffer {
    pub unsafe fn new(width: u32, height: u32, format: PixelBufferFormatType, raw_window_handle: RawWindowHandle) -> Result<PixelBuffer, PixelBufferCreationError> {
        let bit_count = match format {
            PixelBufferFormatType::BGRA => 32,
            PixelBufferFormatType::BGR => 24,
            _ => return Err(PixelBufferCreationError::FormatNotSupported),
        };
        let handle: HBITMAP;
        let bitmap: BITMAP;
        if width != 0 && height != 0 {
            handle = {
                let info = BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as _,
                    biWidth: px_cast(width),
                    biHeight: px_cast(height),
                    biPlanes: 1,
                    biBitCount: bit_count,
                    biCompression: wingdi::BI_RGB,
                    biSizeImage: 0,
                    biXPelsPerMeter: 1,
                    biYPelsPerMeter: 1,
                    biClrUsed: 0,
                    biClrImportant: 0,
                };
                wingdi::CreateDIBSection(
                    winuser::GetDC(ptr::null_mut()),
                    &info as *const BITMAPINFOHEADER as _,
                    wingdi::DIB_RGB_COLORS,
                    &mut ptr::null_mut(),
                    ptr::null_mut(),
                    0,
                )
            };

            assert_ne!(std::ptr::null_mut(), handle);
            bitmap = {
                let mut bitmap: BITMAP = std::mem::zeroed();
                let bytes_written = wingdi::GetObjectW(
                    handle as _,
                    std::mem::size_of::<BITMAP>() as i32,
                    &mut bitmap as *mut BITMAP as *mut _
                );
                assert_ne!(0, bytes_written);
                bitmap
            };
        } else {
            handle = ptr::null_mut();
            bitmap = BITMAP {
                bmType: 0,
                bmWidth: px_cast(width),
                bmHeight: px_cast(height),
                bmWidthBytes: px_cast(width * bit_count as u32 / 8),
                bmPlanes: 1,
                bmBitsPixel: bit_count,
                bmBits: ptr::null_mut(),
            };
        }
        Ok(PixelBuffer {
            handle,
            bitmap,
            len: (bitmap.bmWidthBytes * bitmap.bmHeight) as usize,
            hwnd: hwnd(raw_window_handle),
        })
    }
    pub unsafe fn blit(&self, handle: RawWindowHandle) -> io::Result<()> {
        self.blit_rect((0, 0), (0, 0), (self.width(), self.height()), handle)
    }

    pub unsafe fn blit_rect(&self, src_pos: (u32, u32), dst_pos: (u32, u32), blit_size: (u32, u32), handle: RawWindowHandle) -> io::Result<()> {
        if self.handle == ptr::null_mut() {
            return Ok(());
        }
        let hwnd = hwnd(handle);
        assert_eq!(hwnd, self.hwnd);
        let hdc = winuser::GetDC(hwnd as _);

        let src_dc = wingdi::CreateCompatibleDC(hdc);
        wingdi::SelectObject(src_dc, self.handle as _);
        let result = wingdi::BitBlt(
            hdc,
            px_cast(src_pos.0), px_cast(src_pos.1),
            px_cast(blit_size.0), px_cast(blit_size.1),
            src_dc,
            px_cast(dst_pos.0), px_cast(dst_pos.1),
            wingdi::SRCCOPY,
        );
        let error = io::Error::last_os_error();

        wingdi::DeleteDC(src_dc);

        if result != 0 {
            Ok(())
        } else {
            Err(error)
        }
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.bitmap.bmBitsPixel as usize
    }

    pub fn bytes_per_pixel(&self) -> usize {
        self.bits_per_pixel() / 8
    }

    pub fn width(&self) -> u32 {
        self.bitmap.bmWidth as u32
    }

    pub fn row_len(&self) -> usize {
        self.bitmap.bmWidthBytes as usize
    }

    pub fn height(&self) -> u32 {
        self.bitmap.bmHeight as u32
    }

    fn bytes(&self) -> &[u8] {
        if self.handle == ptr::null_mut() {
            return &[];
        }
        unsafe {
            std::slice::from_raw_parts(
                self.bitmap.bmBits as *const u8,
                self.len
            )
        }
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        if self.handle == ptr::null_mut() {
            return &mut [];
        }
        unsafe {
            std::slice::from_raw_parts_mut(
                self.bitmap.bmBits as *mut u8,
                self.len
            )
        }
    }

    pub fn row(&self, row: u32) -> Option<&[u8]> {
        let index = self.tlo_to_blo(row) as usize * self.row_len();
        let pixel_len = self.width() as usize * self.bytes_per_pixel();
        self.bytes().get(index..index+pixel_len)
    }

    pub fn row_mut(&mut self, row: u32) -> Option<&mut [u8]> {
        let index = self.tlo_to_blo(row) as usize * self.row_len();
        let pixel_len = self.width() as usize * self.bytes_per_pixel();
        self.bytes_mut().get_mut(index..index+pixel_len)
    }

    pub fn rows<'a>(&'a self) -> impl ExactSizeIterator + DoubleEndedIterator<Item=&'a [u8]> {
        let stride = match self.row_len() {
            0 => 1,
            l => l,
        };
        let pixel_len = self.width() as usize * self.bytes_per_pixel();
        self.bytes()
            .chunks(stride)
            .rev()
            .map(move |row| &row[..pixel_len])
    }

    pub fn rows_mut<'a>(&'a mut self) -> impl ExactSizeIterator + DoubleEndedIterator<Item=&'a mut [u8]> {
        let stride = match self.row_len() {
            0 => 1,
            l => l,
        };
        let pixel_len = self.width() as usize * self.bytes_per_pixel();
        self.bytes_mut()
            .chunks_mut(stride)
            .rev()
            .map(move |row| &mut row[..pixel_len])
    }

    #[cfg(feature = "rayon")]
    pub fn par_rows<'a>(&'a self) -> impl IndexedParallelIterator<Item=&'a [u8]> {
        let stride = match self.row_len() {
            0 => 1,
            l => l,
        };
        let pixel_len = self.width() as usize * self.bytes_per_pixel();
        self.bytes()
            .par_chunks(stride)
            .rev()
            .map(move |row| &row[..pixel_len])
    }

    #[cfg(feature = "rayon")]
    pub fn par_rows_mut<'a>(&'a mut self) -> impl IndexedParallelIterator<Item=&'a mut [u8]> {
        let stride = match self.row_len() {
            0 => 1,
            l => l,
        };
        let pixel_len = self.width() as usize * self.bytes_per_pixel();
        self.bytes_mut()
            .par_chunks_mut(stride)
            .rev()
            .map(move |row| &mut row[..pixel_len])
    }

    fn tlo_to_blo(&self, tlo_row: u32) -> u32 {
        self.height() - 1 - tlo_row
    }
}

impl Drop for PixelBuffer {
    fn drop(&mut self) {
        unsafe {
            wingdi::DeleteObject(self.handle as _);
        }
    }
}
