use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winapi::{
    shared::windef::{HBITMAP, HDC, HWND},
    um::{wingdi::{self, BITMAP, BITMAPINFOHEADER}, winuser::{self, PAINTSTRUCT}},
};
use raw_window_handle::{
    HasRawWindowHandle, RawWindowHandle,
    windows::WindowsHandle,
};
use std::{io, ptr, ops::{Deref, DerefMut}};

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();

    let hwnd = match window.raw_window_handle() {
        RawWindowHandle::Windows(WindowsHandle{ hwnd, .. }) => hwnd,
        _ => panic!(),
    };
    let hdc = unsafe{ winuser::GetDC(hwnd as _) };

    let mut dib = false;

    event_loop.run(move |event, _, control_flow| {
        // println!("{:?}", event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput{input: KeyboardInput{state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Space), ..}, ..},
                ..
            } => dib = !dib,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let (width, height): (u32, u32) = window.inner_size().to_physical(window.hidpi_factor()).into();
                if dib {
                    let start = std::time::Instant::now();
                    let mut bitmap = BitmapDIB::new(hdc, width, height);
                    for i in &mut bitmap[..] {
                        *i = 255;
                    }

                    bitmap.blit(hwnd as _).unwrap();
                    let end = std::time::Instant::now();
                    println!("dib {:?}", end - start);
                } else {
                    let buffer= vec![128; (width * height * 4) as usize];
                    let start = std::time::Instant::now();
                    let mut bitmap = BitmapDDB::new(hdc, width, height);

                    bitmap.blit(hwnd as _, &buffer).unwrap();
                    let end = std::time::Instant::now();
                    println!("ddb {:?}", end - start);
                }
            },
            _ => *control_flow = ControlFlow::Wait,
        }
    });
}

struct BitmapDDB {
    handle: HBITMAP,
    bitmap: BITMAP,
    len: usize,
}

impl BitmapDDB {
    pub fn new(hdc: HDC, width: u32, height: u32) -> BitmapDDB {
        let handle = unsafe{ wingdi::CreateCompatibleBitmap(hdc, width as i32, height as i32) };

        assert_ne!(std::ptr::null_mut(), handle);
        let bitmap = unsafe {
            let mut bitmap: BITMAP = std::mem::zeroed();
            let bytes_written = wingdi::GetObjectW(
                handle as _,
                std::mem::size_of::<BITMAP>() as i32,
                &mut bitmap as *mut BITMAP as *mut _
            );
            assert_ne!(0, bytes_written);
            bitmap
        };
        BitmapDDB {
            handle,
            bitmap,
            len: (bitmap.bmWidthBytes * bitmap.bmHeight) as usize
        }
    }

    pub fn blit(&self, hwnd: HWND, buffer: &[u8]) -> io::Result<()> {
        unsafe {
            let mut paint_struct: PAINTSTRUCT = std::mem::zeroed();
            let hdc = winuser::BeginPaint(hwnd, &mut paint_struct);

            wingdi::SetBitmapBits(self.handle, buffer.len() as _, buffer.as_ptr() as _);

            let src_dc = wingdi::CreateCompatibleDC(hdc);
            wingdi::SelectObject(src_dc, self.handle as _);
            let result = wingdi::BitBlt(
                hdc,
                0, 0,
                self.bitmap.bmWidth, self.bitmap.bmHeight,
                src_dc,
                0, 0,
                wingdi::SRCCOPY,
            );
            let error = io::Error::last_os_error();

            winuser::EndPaint(hwnd, &paint_struct);
            wingdi::DeleteDC(src_dc);

            if result != 0 {
                Ok(())
            } else {
                Err(error)
            }
        }
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.bitmap.bmBitsPixel as usize
    }

    pub fn len(&self) -> usize {
        self.len
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

struct BitmapDIB {
    handle: HBITMAP,
    bitmap: BITMAP,
    len: usize,
}

impl BitmapDIB {
    pub fn new(hdc: HDC, width: u32, height: u32) -> BitmapDIB {
        let handle = unsafe{
            let info = BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as _,
                biWidth: width as _,
                biHeight: height as _,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: wingdi::BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 1,
                biYPelsPerMeter: 1,
                biClrUsed: 0,
                biClrImportant: 0,
            };
            wingdi::CreateDIBSection(
                hdc,
                &info as *const BITMAPINFOHEADER as _,
                wingdi::DIB_RGB_COLORS,
                &mut ptr::null_mut(),
                ptr::null_mut(),
                0,
            )
        };

        assert_ne!(std::ptr::null_mut(), handle);
        let bitmap = unsafe {
            let mut bitmap: BITMAP = std::mem::zeroed();
            let bytes_written = wingdi::GetObjectW(
                handle as _,
                std::mem::size_of::<BITMAP>() as i32,
                &mut bitmap as *mut BITMAP as *mut _
            );
            assert_ne!(0, bytes_written);
            bitmap
        };
        BitmapDIB {
            handle,
            bitmap,
            len: (bitmap.bmWidthBytes * bitmap.bmHeight) as usize
        }
    }

    pub fn blit(&self, hwnd: HWND) -> io::Result<()> {
        unsafe {
            let mut paint_struct: PAINTSTRUCT = std::mem::zeroed();
            let hdc = winuser::BeginPaint(hwnd, &mut paint_struct);

            let src_dc = wingdi::CreateCompatibleDC(hdc);
            wingdi::SelectObject(src_dc, self.handle as _);
            let result = wingdi::BitBlt(
                hdc,
                0, 0,
                self.bitmap.bmWidth, self.bitmap.bmHeight,
                src_dc,
                0, 0,
                wingdi::SRCCOPY,
            );
            let error = io::Error::last_os_error();

            winuser::EndPaint(hwnd, &paint_struct);
            wingdi::DeleteDC(src_dc);

            if result != 0 {
                Ok(())
            } else {
                Err(error)
            }
        }
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.bitmap.bmBitsPixel as usize
    }

    pub fn len(&self) -> usize {
        self.len
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

impl Deref for BitmapDIB {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.bitmap.bmBits as *const u8,
                self.len()
            )
        }
    }
}

impl DerefMut for BitmapDIB {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.bitmap.bmBits as *mut u8,
                self.len()
            )
        }
    }
}
