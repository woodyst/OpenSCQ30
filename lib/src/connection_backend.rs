use crate::api::connection::{self, RfcommBackend};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod none;

#[cfg(test)]
pub(crate) mod mock;

/// Groups together platform specific implementations of various means of connecting to devices.
pub trait ConnectionBackends {
    type Rfcomm: RfcommBackend + Send + Sync;

    fn rfcomm(&self) -> impl Future<Output = connection::Result<Self::Rfcomm>> + Send;
}

#[cfg(target_os = "linux")]
pub fn default_backends() -> Option<impl ConnectionBackends> {
    Some(linux::PlatformConnectionBackends)
}

#[cfg(target_os = "windows")]
pub fn default_backends() -> Option<impl ConnectionBackends> {
    Some(windows::PlatformConnectionBackends)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn default_backends() -> Option<impl ConnectionBackends> {
    None::<none::NoneConnectionBackends>
}
