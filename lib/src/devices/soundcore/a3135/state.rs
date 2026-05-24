use openscq30_lib_macros::Has;

use crate::devices::soundcore::{
    a3135,
    common::{
        state::Update,
        structures::{AutoPowerOff, BatteryLevel, EqualizerConfiguration, Ldac, SerialNumber, VoicePrompt},
    },
};

use super::packets::inbound::{A3135EqPacket, A3135StateUpdatePacket};

#[derive(Debug, Clone, PartialEq, Eq, Has)]
pub struct A3135State {
    battery_level: BatteryLevel,
    volume: a3135::structures::Volume,
    ldac: Ldac,
    led_brightness: a3135::structures::LedBrightness,
    auto_power_off: AutoPowerOff,
    voice_prompt: VoicePrompt,
    adaptive_direction: a3135::structures::AdaptiveDirection,
    power_off_pending: a3135::structures::PowerOffPending,
    equalizer_configuration: EqualizerConfiguration<1, 9, -60, 60, 1>,
    firmware_version: a3135::structures::A3135FirmwareVersion,
    serial_number: SerialNumber,
}

impl A3135State {
    pub fn new(
        packet: A3135StateUpdatePacket,
        ldac: Ldac,
        led_brightness: a3135::structures::LedBrightness,
        eq_packet: A3135EqPacket,
    ) -> Self {
        Self {
            battery_level: packet.battery_level,
            volume: packet.volume,
            ldac,
            led_brightness,
            adaptive_direction: Default::default(),
            auto_power_off: AutoPowerOff::default(),
            voice_prompt: packet.voice_prompt,
            power_off_pending: Default::default(),
            equalizer_configuration: eq_packet.equalizer_configuration,
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
            voice_prompt: _,
            firmware_version,
            serial_number,
        } = partial;
        self.volume = volume;
        self.battery_level = battery_level;
        self.firmware_version = firmware_version;
        self.serial_number = serial_number;
    }
}
