use openscq30_lib_macros::Has;

use crate::devices::soundcore::{
    a3961::{packets::inbound::A3961StateUpdatePacket, structures::A3961SoundModes},
    common::structures::{DualBattery, DualFirmwareVersion, SerialNumber, TwsStatus},
};

#[derive(Debug, Clone, PartialEq, Eq, Has)]
pub struct A3961State {
    tws_status: TwsStatus,
    battery: DualBattery,
    firmware_version: DualFirmwareVersion,
    serial_number: SerialNumber,
    sound_modes: A3961SoundModes,
}

impl From<A3961StateUpdatePacket> for A3961State {
    fn from(value: A3961StateUpdatePacket) -> Self {
        Self {
            tws_status: value.tws_status,
            battery: value.battery,
            firmware_version: value.firmware_version,
            serial_number: value.serial_number,
            sound_modes: value.sound_modes,
        }
    }
}
