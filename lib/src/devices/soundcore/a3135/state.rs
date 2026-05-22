use openscq30_lib_macros::Has;

use crate::devices::soundcore::{
    a3135,
    common::{
        state::Update,
        structures::{AutoPowerOff, BatteryLevel, Ldac, SerialNumber, VoicePrompt},
    },
};

use super::packets::inbound::A3135StateUpdatePacket;

#[derive(Debug, Clone, PartialEq, Eq, Has)]
pub struct A3135State {
    battery_level: BatteryLevel,
    volume: a3135::structures::Volume,
    ldac: Ldac,
    auto_power_off: AutoPowerOff,
    voice_prompt: VoicePrompt,
    power_off_pending: a3135::structures::PowerOffPending,
    firmware_version: a3135::structures::A3135FirmwareVersion,
    serial_number: SerialNumber,
}

impl A3135State {
    pub fn new(packet: A3135StateUpdatePacket, ldac: Ldac) -> Self {
        Self {
            battery_level: packet.battery_level,
            volume: packet.volume,
            ldac,
            auto_power_off: AutoPowerOff::default(),
            voice_prompt: VoicePrompt::default(),
            power_off_pending: Default::default(),
            firmware_version: packet.firmware_version,
            serial_number: packet.serial_number,
        }
    }
}

impl Update<A3135StateUpdatePacket> for A3135State {
    fn update(&mut self, partial: A3135StateUpdatePacket) {
        let A3135StateUpdatePacket {
            volume,
            battery_level,
            firmware_version,
            serial_number,
        } = partial;
        self.volume = volume;
        self.battery_level = battery_level;
        self.firmware_version = firmware_version;
        self.serial_number = serial_number;
    }
}
