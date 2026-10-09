#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod other;

#[cfg(target_os = "macos")]
pub use macos::{configure_overlay, focus_overlay, plugin};
#[cfg(not(target_os = "macos"))]
pub use other::{configure_overlay, focus_overlay, plugin};
