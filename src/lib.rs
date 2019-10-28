mod platform_impl;
use raw_window_handle::HasRawWindowHandle;
use std::{io, ops::{Deref, DerefMut}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelBufferFormat {
    BGR,
    BGRA,
    RGB,
    RGBA,
}

#[derive(Debug, Clone)]
pub enum PixelBufferCreationError {
    FormatNotSupported,
}

pub struct PixelBuffer {
    p: platform_impl::PixelBuffer,
}

impl PixelBuffer {
    pub fn new<H: HasRawWindowHandle>(width: u32, height: u32, format: PixelBufferFormat, window: &H) -> Result<PixelBuffer, PixelBufferCreationError> {
        unsafe {
            platform_impl::PixelBuffer::new(width, height, format, window.raw_window_handle()).map(|p| PixelBuffer { p })
        }
    }
    pub fn blit<H: HasRawWindowHandle>(&self, window: &H) -> io::Result<()> {
        unsafe {
            self.p.blit(window.raw_window_handle())
        }
    }

    pub fn blit_rect<H: HasRawWindowHandle>(&self, src_pos: (u32, u32), dst_pos: (u32, u32), blit_size: (u32, u32), window: &H) -> io::Result<()> {
        unsafe {
            self.p.blit_rect(src_pos, dst_pos, blit_size, window.raw_window_handle())
        }
    }

    pub fn bits_per_pixel(&self) -> usize {
        self.p.bits_per_pixel()
    }

    pub fn width(&self) -> u32 {
        self.p.width()
    }

    pub fn width_bytes(&self) -> usize {
        self.p.width_bytes()
    }

    pub fn height(&self) -> u32 {
        self.p.height()
    }
}

impl Deref for PixelBuffer {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.p.deref()
    }
}

impl DerefMut for PixelBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.p.deref_mut()
    }
}
