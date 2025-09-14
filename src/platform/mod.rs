mod icon;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::run_tray;

#[cfg(not(target_os = "macos"))]
mod default;
#[cfg(not(target_os = "macos"))]
pub use self::default::run_tray;
