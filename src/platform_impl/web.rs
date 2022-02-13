use std::io;
use std::io::ErrorKind;

use raw_window_handle::RawWindowHandle;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;
use web_sys::ImageData;

use crate::PixelBufferCreationError;
use crate::PixelBufferFormatSupported;
use crate::PixelBufferFormatType;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

impl PixelBufferFormatSupported for crate::RGBA {}
pub type NativeFormat = crate::RGBA;

pub struct PixelBuffer {
    buffer: Box<[u8]>,
    width: u32,
}

impl PixelBuffer {
    pub unsafe fn new(
        width: u32,
        height: u32,
        format: PixelBufferFormatType,
        _: RawWindowHandle,
    ) -> Result<PixelBuffer, PixelBufferCreationError> {
        if format != PixelBufferFormatType::RGBA {
            return Err(PixelBufferCreationError::FormatNotSupported);
        }

        Ok(Self {
            // This only runs on WebAssembly, so `usize` will always just be a `u32` (or possibly `u64` if/when wasm64 becomes a thing).
            buffer: vec![0; 4 * width as usize * height as usize].into_boxed_slice(),
            width,
        })
    }
    pub unsafe fn blit(&self, handle: RawWindowHandle) -> io::Result<()> {
        self.blit_rect((0, 0), (0, 0), (self.width(), self.height()), handle)
    }

    pub unsafe fn blit_rect(
        &self,
        src_pos: (u32, u32),
        dst_pos: (u32, u32),
        blit_size: (u32, u32),
        handle: RawWindowHandle,
    ) -> io::Result<()> {
        // This should only throw an error if the buffer we pass's size is incorrect, which is impossible.
        let data = ImageData::new_with_u8_clamped_array(Clamped(&self.buffer), self.width).unwrap();

        let ctx = get_context(handle)?;

        ctx.put_image_data_with_dirty_x_and_dirty_y_and_dirty_width_and_dirty_height(
            &data,
            dst_pos.0 as f64,
            dst_pos.1 as f64,
            src_pos.0 as f64,
            src_pos.1 as f64,
            blit_size.0 as f64,
            blit_size.1 as f64,
        )
        // This can only throw an error if `data` is detached, which is impossible.
        .unwrap();

        Ok(())
    }

    pub fn bits_per_pixel(&self) -> usize {
        8 * self.bytes_per_pixel()
    }

    pub fn bytes_per_pixel(&self) -> usize {
        4
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn row_len(&self) -> usize {
        4 * self.width as usize
    }

    pub fn height(&self) -> u32 {
        self.buffer.len() as u32 / (4 * self.width)
    }

    pub fn row(&self, row: u32) -> Option<&[u8]> {
        let start = row as usize * self.row_len();
        self.buffer.get(start..start + self.row_len())
    }

    pub fn row_mut(&mut self, row: u32) -> Option<&mut [u8]> {
        let start = row as usize * self.row_len();
        self.buffer.get_mut(start..start + self.row_len())
    }

    pub fn rows<'a>(&'a self) -> impl ExactSizeIterator + DoubleEndedIterator<Item = &'a [u8]> {
        self.buffer.chunks(self.row_len())
    }

    pub fn rows_mut<'a>(
        &'a mut self,
    ) -> impl ExactSizeIterator + DoubleEndedIterator<Item = &'a mut [u8]> {
        self.buffer.chunks_mut(self.row_len())
    }

    #[cfg(feature = "rayon")]
    pub fn par_rows<'a>(&'a self) -> impl IndexedParallelIterator<Item = &'a [u8]> {
        self.buffer.par_chunks(self.row_len())
    }

    #[cfg(feature = "rayon")]
    pub fn par_rows_mut<'a>(&'a mut self) -> impl IndexedParallelIterator<Item = &'a mut [u8]> {
        self.buffer.par_chunks_mut(self.row_len())
    }
}

fn get_context(handle: RawWindowHandle) -> io::Result<CanvasRenderingContext2d> {
    let id = match handle {
        RawWindowHandle::Web(handle) => handle.id,
        _ => {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Unknown kind of `RawWindowHandle`",
            ))
        }
    };

    let canvas: HtmlCanvasElement = web_sys::window()
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::Unsupported,
                "`window` is not present in this runtime",
            )
        })?
        .document()
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::Unsupported,
                "`document` is not present in this runtime",
            )
        })?
        .query_selector(&format!("canvas[data-raw-handle=\"{}\"]", id))
        // `querySelector` only throws an error if the selector is invalid.
        .unwrap()
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "No canvas found with the given id"))?
        // We already made sure this was a canvas in `querySelector`.
        .unchecked_into();

    Ok(canvas
        .get_context("2d")
        .map_err(|_| {
            io::Error::new(
                ErrorKind::Other,
                "Canvas already controlled using `OffscreenCanvas`",
            )
        })?
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::Other,
                "A canvas context other than `CanvasRenderingContext2d` was already created",
            )
        })?
        .dyn_into()
        .expect("`getContext(\"2d\") didn't return a `CanvasRenderingContext2d`"))
}
