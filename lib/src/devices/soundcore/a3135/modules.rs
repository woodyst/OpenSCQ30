use async_trait::async_trait;
use openscq30_lib_has::Has;
use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

use crate::{
    api::settings::{CategoryId, Setting, SettingId, Value},
    devices::soundcore::{
        a3135,
        common::{
            device::SoundcoreDeviceBuilder,
            modules::ModuleCollection,
            settings_manager::{SettingHandler, SettingHandlerError, SettingHandlerResult},
            structures::SerialNumber,
        },
    },
    macros::enum_subset,
};

use super::structures::A3135FirmwareVersion;

mod adaptive_direction;
mod brightness;
pub mod equalizer;
mod power_off;
mod volume;

enum_subset!(
    SettingId,
    #[derive(EnumString, EnumIter, IntoStaticStr)]
    enum A3135SerialAndFirmwareSetting {
        SerialNumber,
        FirmwareVersion,
    }
);

#[derive(Default)]
struct A3135SerialAndFirmwareHandler;

#[async_trait]
impl<T> SettingHandler<T> for A3135SerialAndFirmwareHandler
where
    T: Has<SerialNumber> + Has<A3135FirmwareVersion> + Send,
{
    fn settings(&self) -> Vec<SettingId> {
        A3135SerialAndFirmwareSetting::iter()
            .map(Into::into)
            .collect()
    }

    fn get(&self, state: &T, setting_id: &SettingId) -> Option<Setting> {
        let serial: &SerialNumber = state.get();
        let firmware: &A3135FirmwareVersion = state.get();
        let setting: A3135SerialAndFirmwareSetting = (*setting_id).try_into().ok()?;
        Some(match setting {
            A3135SerialAndFirmwareSetting::SerialNumber => Setting::Information {
                value: serial.to_string(),
                translated_value: serial.to_string(),
            },
            A3135SerialAndFirmwareSetting::FirmwareVersion => Setting::Information {
                value: firmware.to_string(),
                translated_value: firmware.to_string(),
            },
        })
    }

    async fn set(
        &self,
        _state: &mut T,
        _setting_id: &SettingId,
        _value: Value,
    ) -> SettingHandlerResult<()> {
        Err(SettingHandlerError::ReadOnly)
    }
}

impl<StateType> ModuleCollection<StateType>
where
    StateType: Has<SerialNumber> + Has<A3135FirmwareVersion> + Clone + Send + Sync,
{
    pub fn add_a3135_serial_number_and_firmware_version(&mut self) {
        self.setting_manager.add_handler(
            CategoryId::DeviceInformation,
            A3135SerialAndFirmwareHandler,
        );
    }
}

impl<StateType> SoundcoreDeviceBuilder<StateType>
where
    StateType: Has<SerialNumber> + Has<A3135FirmwareVersion> + Send + Sync + Clone + 'static,
{
    pub fn a3135_serial_number_and_firmware_version(&mut self) {
        self.module_collection()
            .add_a3135_serial_number_and_firmware_version();
    }
}

impl<StateType> SoundcoreDeviceBuilder<StateType>
where
    StateType: Has<a3135::structures::Volume> + Send + Sync + Clone + 'static,
{
    pub fn a3135_volume(&mut self, max_volume: u8) {
        let packet_io = self.packet_io_controller().clone();
        self.module_collection()
            .add_a3135_volume(packet_io, max_volume);
    }
}

impl<StateType> SoundcoreDeviceBuilder<StateType>
where
    StateType: Has<a3135::structures::PowerOffPending> + Send + Sync + Clone + 'static,
{
    pub fn a3135_power_off(&mut self) {
        let packet_io = self.packet_io_controller().clone();
        self.module_collection().add_a3135_power_off(packet_io);
    }
}

impl<StateType> SoundcoreDeviceBuilder<StateType>
where
    StateType: Has<a3135::structures::LedBrightness> + Send + Sync + Clone + 'static,
{
    pub fn a3135_brightness(&mut self) {
        let packet_io = self.packet_io_controller().clone();
        self.module_collection().add_a3135_brightness(packet_io);
    }
}

impl<StateType> SoundcoreDeviceBuilder<StateType>
where
    StateType: Has<a3135::structures::AdaptiveDirection> + Send + Sync + Clone + 'static,
{
    pub fn a3135_adaptive_direction_readonly(&mut self) {
        self.module_collection().add_a3135_adaptive_direction();
    }
}
