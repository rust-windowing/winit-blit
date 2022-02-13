pub use self::platform_impl::*;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod platform_impl;

#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
mod platform_impl;
