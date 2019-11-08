mod platform_impl;
use raw_window_handle::HasRawWindowHandle;
use std::{
    io,
    fmt::Debug,
    marker::PhantomData,
};

#[derive(Debug, Clone)]
pub enum PixelBufferCreationError {
    FormatNotSupported,
}

/// A buffer of pixels that can be blitted onto a window.
///
/// The pixel buffer's origin is in the top-left corner of the image.
pub struct PixelBuffer {
    p: platform_impl::PixelBuffer,
}

/// A buffer of pixels with a statically-checked pixel format.
///
/// The pixel buffer's origin is in the top-left corner of the image.
pub struct PixelBufferTyped<P: PixelBufferFormat> {
    p: PixelBuffer,
    _format: PhantomData<P>
}

impl PixelBufferFormatType {
    /// The native pixel buffer format for the current plaform.
    pub const NATIVE: PixelBufferFormatType = NativeFormat::FORMAT_TYPE;
}

impl PixelBuffer {
    /// Initialize a new pixel buffer.
    ///
    /// Can return `Err` if the platform doesn't support the requested pixel buffer type.
    pub fn new<H: HasRawWindowHandle>(width: u32, height: u32, format: PixelBufferFormatType, window: &H) -> Result<PixelBuffer, PixelBufferCreationError> {
        unsafe {
            platform_impl::PixelBuffer::new(width, height, format, window.raw_window_handle()).map(|p| PixelBuffer { p })
        }
    }

    /// Blits the pixel buffer's contents onto `window`.
    ///
    /// # Panics
    /// The `window` passed to this function must be the same `window` passed to `new`. Failing to
    /// do so will result in a panic.
    pub fn blit<H: HasRawWindowHandle>(&self, window: &H) -> io::Result<()> {
        unsafe {
            self.p.blit(window.raw_window_handle())
        }
    }

    /// Blits a subsection of the pixel buffer's contents onto `window`.
    ///
    /// # Panics
    /// The `window` passed to this function must be the same `window` passed to `new`. Failing to
    /// do so will result in a panic.
    pub fn blit_rect<H: HasRawWindowHandle>(&self, src_pos: (u32, u32), dst_pos: (u32, u32), blit_size: (u32, u32), window: &H) -> io::Result<()> {
        unsafe {
            self.p.blit_rect(src_pos, dst_pos, blit_size, window.raw_window_handle())
        }
    }

    /// The total number of bits in an individual pixel.
    ///
    /// Will always be a multiple of `8`.
    pub fn bits_per_pixel(&self) -> usize {
        self.p.bits_per_pixel()
    }

    /// The total number of bytes in an individual pixel.
    pub fn bytes_per_pixel(&self) -> usize {
        self.p.bytes_per_pixel()
    }

    /// The width, in pixels, of the pixel buffer.
    pub fn width(&self) -> u32 {
        self.p.width()
    }

    /// The length, in bytes, of a single row of the pixel buffer.
    pub fn row_len(&self) -> usize {
        self.p.row_len()
    }

    /// The height, in pixels, of the pixel buffer.
    pub fn height(&self) -> u32 {
        self.p.height()
    }

    /// Gets the row at the particular height.
    pub fn row(&self, row: u32) -> Option<&[u8]> {
        self.p.row(row)
    }

    /// Mutably gets the row at the particular height.
    pub fn row_mut(&mut self, row: u32) -> Option<&mut [u8]> {
        self.p.row_mut(row)
    }

    /// Iterate through all rows in the pixel buffer.
    pub fn rows<'a>(&'a self) -> impl Iterator<Item=&'a [u8]> {
        self.p.rows()
    }

    /// Mutably iterate through all rows in the pixel buffer.
    pub fn rows_mut<'a>(&'a mut self) -> impl Iterator<Item=&'a mut [u8]> {
        self.p.rows_mut()
    }
}

impl<P: PixelBufferFormat> PixelBufferTyped<P> {
    /// Initialize a new pixel buffer.
    ///
    /// Can return `Err` if the platform doesn't support the requested pixel buffer type.
    pub fn new<H: HasRawWindowHandle>(width: u32, height: u32, window: &H) -> Result<PixelBufferTyped<P>, PixelBufferCreationError> {
        Ok(PixelBufferTyped {
            p: PixelBuffer::new(width, height, P::FORMAT_TYPE, window)?,
            _format: PhantomData,
        })
    }

    /// Initialize a new pixel buffer.
    ///
    /// This always works, since we've statically checked that the pixel format is supported by
    /// the platform.
    pub fn new_supported<H: HasRawWindowHandle>(width: u32, height: u32, window: &H) -> PixelBufferTyped<P>
        where P: PixelBufferFormatSupported
    {
        Self::new(width, height, window).unwrap()
    }

    /// Blits the pixel buffer's contents onto `window`.
    ///
    /// # Panics
    /// The `window` passed to this function must be the same `window` passed to `new`. Failing to
    /// do so will result in a panic.
    pub fn blit<H: HasRawWindowHandle>(&self, window: &H) -> io::Result<()> {
        self.p.blit(window)
    }

    /// Blits a subsection of the pixel buffer's contents onto `window`.
    ///
    /// # Panics
    /// The `window` passed to this function must be the same `window` passed to `new`. Failing to
    /// do so will result in a panic.
    pub fn blit_rect<H: HasRawWindowHandle>(&self, src_pos: (u32, u32), dst_pos: (u32, u32), blit_size: (u32, u32), window: &H) -> io::Result<()> {
        self.p.blit_rect(src_pos, dst_pos, blit_size, window)
    }

    /// The total number of bits in an individual pixel.
    ///
    /// Will always be a multiple of `8`.
    pub fn bits_per_pixel(&self) -> usize {
        self.p.bits_per_pixel()
    }

    /// The total number of bytes in an individual pixel.
    pub fn bytes_per_pixel(&self) -> usize {
        self.p.bytes_per_pixel()
    }

    /// The width, in pixels, of the pixel buffer.
    pub fn width(&self) -> u32 {
        self.p.width()
    }

    /// The length, in bytes, of a single row of the pixel buffer.
    pub fn row_len(&self) -> usize {
        self.p.row_len()
    }

    /// The height, in pixels, of the pixel buffer.
    pub fn height(&self) -> u32 {
        self.p.height()
    }

    /// Gets the row at the particular height.
    pub fn row(&self, row: u32) -> Option<&[P]> {
        self.p.row(row).map(P::from_raw_slice)
    }

    /// Mutably gets the row at the particular height.
    pub fn row_mut(&mut self, row: u32) -> Option<&mut [P]> {
        self.p.row_mut(row).map(P::from_raw_slice_mut)
    }

    /// Iterate through all rows in the pixel buffer.
    pub fn rows<'a>(&'a self) -> impl Iterator<Item=&'a [P]> {
        self.p.rows().map(P::from_raw_slice)
    }

    /// Mutably iterate through all rows in the pixel buffer.
    pub fn rows_mut<'a>(&'a mut self) -> impl Iterator<Item=&'a mut [P]> {
        self.p.rows_mut().map(P::from_raw_slice_mut)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelBufferFormatType {
    BGR,
    BGRA,
    RGB,
    RGBA,
}

pub trait PixelBufferFormatSupported: PixelBufferFormat {}
pub unsafe trait PixelBufferFormat: Sized + Debug + Copy + AsRef<<Self as PixelBufferFormat>::Array> + AsMut<<Self as PixelBufferFormat>::Array> {
    type Array: Debug + Copy + AsRef<[u8]> + AsMut<[u8]> + AsRef<Self> + AsMut<Self>;
    const DEFAULT: Self;
    const FORMAT_TYPE: PixelBufferFormatType;

    fn from_rgb(r: u8, g: u8, b: u8) -> Self;
    fn from_raw_slice(raw: &[u8]) -> &[Self];
    fn from_raw_slice_mut(raw: &mut [u8]) -> &mut [Self];
    fn to_raw_slice(slice: &[Self]) -> &[u8];
    fn to_raw_slice_mut(slice: &mut [Self]) -> &mut [u8];
}

macro_rules! pixel_buffer_format {
    ($pixel:ident($($c:ident),+): $array:ty = $default:expr) => {
        #[repr(C)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $pixel {
            $(pub $c: u8),+
        }
        impl $pixel {
            pub const DEFAULT: $pixel = $default;
            pub const FORMAT_TYPE: PixelBufferFormatType = PixelBufferFormatType::$pixel;
            #[inline(always)]
            fn size() -> usize {
                use std::mem;
                let size = mem::size_of::<Self>() / mem::size_of::<u8>();
                assert_eq!(0, mem::size_of::<Self>() % mem::size_of::<u8>());
                size
            }

            pub const fn new($($c: u8),+) -> $pixel {
                $pixel {
                    $($c),+
                }
            }
            pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
                Self {
                    r, g, b,
                    ..Self::DEFAULT
                }
            }
            #[inline(always)]
            pub fn from_raw_slice(raw: &[u8]) -> &[Self] {
                let size = Self::size();
                assert_eq!(
                    0,
                    raw.len() % size,
                    "raw slice length not multiple of {}",
                    size
                );
                unsafe { ::std::slice::from_raw_parts(raw.as_ptr() as *const Self, raw.len() / size) }
            }

            #[inline(always)]
            pub fn from_raw_slice_mut(raw: &mut [u8]) -> &mut [Self] {
                let size = Self::size();
                assert_eq!(
                    0,
                    raw.len() % size,
                    "raw slice length not multiple of {}",
                    size
                );
                unsafe {
                    ::std::slice::from_raw_parts_mut(raw.as_mut_ptr() as *mut Self, raw.len() / size)
                }
            }

            #[inline(always)]
            pub fn to_raw_slice(slice: &[Self]) -> &[u8] {
                let size = Self::size();
                unsafe {
                    ::std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len() * size)
                }
            }

            #[inline(always)]
            pub fn to_raw_slice_mut(slice: &mut [Self]) -> &mut [u8] {
                let size = Self::size();
                unsafe {
                    ::std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, slice.len() * size)
                }
            }
        }
        unsafe impl PixelBufferFormat for $pixel {
            type Array = $array;
            const DEFAULT: Self = Self::DEFAULT;
            const FORMAT_TYPE: PixelBufferFormatType = Self::FORMAT_TYPE;

            fn from_rgb(r: u8, g: u8, b: u8) -> Self {
                Self::from_rgb(r, g, b)
            }
            #[inline(always)]
            fn from_raw_slice(raw: &[u8]) -> &[Self] {
                Self::from_raw_slice(raw)
            }

            #[inline(always)]
            fn from_raw_slice_mut(raw: &mut [u8]) -> &mut [Self] {
                Self::from_raw_slice_mut(raw)
            }

            #[inline(always)]
            fn to_raw_slice(slice: &[Self]) -> &[u8] {
                Self::to_raw_slice(slice)
            }

            #[inline(always)]
            fn to_raw_slice_mut(slice: &mut [Self]) -> &mut [u8] {
                Self::to_raw_slice_mut(slice)
            }
        }

        impl AsRef<$array> for $pixel {
            fn as_ref(&self) -> &$array {
                unsafe{ &*(self as *const Self as *const $array) }
            }
        }
        impl AsMut<$array> for $pixel {
            fn as_mut(&mut self) -> &mut $array {
                unsafe{ &mut *(self as *mut Self as *mut $array) }
            }
        }
        impl AsRef<$pixel> for $array {
            fn as_ref(&self) -> &$pixel {
                unsafe{ &*(self as *const Self as *const $pixel) }
            }
        }
        impl AsMut<$pixel> for $array {
            fn as_mut(&mut self) -> &mut $pixel {
                unsafe{ &mut *(self as *mut Self as *mut $pixel) }
            }
        }
        impl From<$array> for $pixel {
            fn from(array: $array) -> $pixel {
                unsafe{ std::mem::transmute(array) }
            }
        }
        impl From<$pixel> for $array {
            fn from(array: $pixel) -> $array {
                unsafe{ std::mem::transmute(array) }
            }
        }
        impl Default for $pixel {
            fn default() -> Self {
                Self::DEFAULT
            }
        }
    };
}

pub type NativeFormat = platform_impl::NativeFormat;
pixel_buffer_format!(BGR(b, g, r): [u8; 3] = Self::new(0, 0, 0));
pixel_buffer_format!(BGRA(b, g, r, a): [u8; 4] = Self::new(0, 0, 0, 255));
pixel_buffer_format!(RGB(r, g, b): [u8; 3] = Self::new(0, 0, 0));
pixel_buffer_format!(RGBA(r, g, b, a): [u8; 4] = Self::new(0, 0, 0, 255));
