#[cfg(feature = "agnostic")]
pub use winit_agnostic as platform;
#[cfg(all(not(feature = "agnostic"), android_platform))]
pub(crate) use winit_android as platform;
#[cfg(all(not(feature = "agnostic"), macos_platform))]
pub(crate) use winit_appkit as platform;
#[cfg(all(not(feature = "agnostic"), wayland_platform))]
mod linux;
#[cfg(all(not(feature = "agnostic"), orbital_platform))]
pub(crate) use winit_orbital as platform;
#[cfg(all(not(feature = "agnostic"), ios_platform))]
pub(crate) use winit_uikit as platform;
#[cfg(all(not(feature = "agnostic"), web_platform))]
pub(crate) use winit_web as platform;
#[cfg(all(not(feature = "agnostic"), windows_platform))]
pub(crate) use winit_win32 as platform;

#[cfg(any(x11_platform, wayland_platform))]
use self::linux as platform;
#[allow(unused_imports)]
pub use self::platform::*;

#[cfg(all(
    not(ios_platform),
    not(windows_platform),
    not(macos_platform),
    not(android_platform),
    not(x11_platform),
    not(wayland_platform),
    not(web_platform),
    not(orbital_platform),
))]
compile_error!("The platform you're compiling for is not supported by winit");
