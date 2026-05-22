use openscq30_lib_macros::Has;

use crate::devices::soundcore::{
    a3135,
    common::{
        state::Update,
        structures::{BatteryLevel, SerialNumber},
    },
};

use super::packets::inbound::A3135StateUpdatePacket;

#[derive(Debug, Clone, PartialEq, Eq, Has)]
pub struct A3135State {
    battery_level: BatteryLevel,
    firmware_version: a3135::structures::A3135FirmwareVersion,
    serial_number: SerialNumber,
}

impl A3135State {
    pub fn new(packet: A3135StateUpdatePacket) -> Self {
        Self {
            battery_level: packet.battery_level,
            firmware_version: packet.firmware_version,
            serial_number: packet.serial_number,
        }
    }
}

impl Update<A3135StateUpdatePacket> for A3135State {
    fn update(&mut self, partial: A3135StateUpdatePacket) {
        let A3135StateUpdatePacket {
            battery_level,
            firmware_version,
            serial_number,
        } = partial;
        self.battery_level = battery_level;
        self.firmware_version = firmware_version;
        self.serial_number = serial_number;
    }
}
