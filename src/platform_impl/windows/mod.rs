use crate::{PixelBufferFormat, PixelBufferCreationError};
use winapi::{
    shared::windef::{HBITMAP},
    um::{wingdi::{self, BITMAP, BITMAPINFOHEADER}, winuser},
};
use raw_window_handle::{RawWindowHandle, windows::WindowsHandle};
use std::{convert::TryInto, ptr, io, ops::{Deref, DerefMut}};

pub struct PixelBuffer {
    handle: HBITMAP,
    bitmap: BITMAP,
    len: usize,
}

fn px_cast(u: u32) -> i32 {
    u.try_into().expect("Pixel value too large; must be less than 2,147,483,647")
}

impl PixelBuffer {
    pub unsafe fn new(width: u32, height: u32, format: PixelBufferFormat, _: RawWindowHandle) -> Result<PixelBuffer, PixelBufferCreationError> {
        let bit_count = match format {
            PixelBufferFormat::BGRA => 32,
            PixelBufferFormat::BGR => 24,
            _ => return Err(PixelBufferCreationError::FormatNotSupported),
        };
        let handle = {
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
        let bitmap = {
            let mut bitmap: BITMAP = std::mem::zeroed();
            let bytes_written = wingdi::GetObjectW(
                handle as _,
                std::mem::size_of::<BITMAP>() as i32,
                &mut bitmap as *mut BITMAP as *mut _
            );
            assert_ne!(0, bytes_written);
            bitmap
        };
        Ok(PixelBuffer {
            handle,
            bitmap,
            len: (bitmap.bmWidthBytes * bitmap.bmHeight) as usize
        })
    }
    pub unsafe fn blit(&self, handle: RawWindowHandle) -> io::Result<()> {
        self.blit_rect((0, 0), (0, 0), (self.width(), self.height()), handle)
    }

    pub unsafe fn blit_rect(&self, src_pos: (u32, u32), dst_pos: (u32, u32), blit_size: (u32, u32), handle: RawWindowHandle) -> io::Result<()> {
        let hwnd = match handle {
            RawWindowHandle::Windows(WindowsHandle{hwnd, ..}) => hwnd,
            _ => panic!("Unsupported window handle type"),
        };
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

    pub fn width(&self) -> u32 {
        self.bitmap.bmWidth as u32
    }

    pub fn width_bytes(&self) -> usize {
        self.bitmap.bmWidthBytes as usize
    }

    pub fn height(&self) -> u32 {
        self.bitmap.bmHeight as u32
    }
}

impl Drop for PixelBuffer {
    fn drop(&mut self) {
        unsafe {
            wingdi::DeleteObject(self.handle as _);
        }
    }
}

impl Deref for PixelBuffer {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.bitmap.bmBits as *const u8,
                self.len
            )
        }
    }
}

impl DerefMut for PixelBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.bitmap.bmBits as *mut u8,
                self.len
            )
        }
    }
}
