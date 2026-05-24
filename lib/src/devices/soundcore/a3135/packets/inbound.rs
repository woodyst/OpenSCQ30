use std::{array, iter};

use async_trait::async_trait;
use nom::{
    IResult, Parser,
    bytes::complete::take,
    combinator::map,
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};
use tokio::sync::watch;

use crate::{
    api::device,
    devices::soundcore::{
        a3135::{self, state::A3135State},
        common::{
            modules::ModuleCollection,
            packet::{
                self, Command,
                inbound::{FromPacketBody, TryToPacket},
                outbound::{ToPacket, REQUEST_LDAC_STATE_COMMAND},
            },
            packet_manager::PacketHandler,
            state::Update,
            structures::{BatteryLevel, EqualizerConfiguration, Ldac, SerialNumber, VoicePrompt, VolumeAdjustments},
        },
    },
};

pub const REQUEST_BRIGHTNESS_COMMAND: packet::Command = packet::Command([0x10, 0x93]);

/// Body layout (29 bytes):
///   [0]       volume (0–31)
///   [1]       battery level (0–5)
///   [2]       unknown
///   [3]       unknown
///   [4]       voice prompt (0=off, 1=on)
///   [5]       unknown
///   [6]       unknown (possibly auto power off duration)
///   [7..12]   firmware version ASCII "X.Y.Z" (5 bytes)
///   [12..28]  serial number ASCII (16 bytes)
///   [28]      unknown (1 byte)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct A3135StateUpdatePacket {
    pub volume: a3135::structures::Volume,
    pub battery_level: BatteryLevel,
    pub voice_prompt: VoicePrompt,
    pub firmware_version: a3135::structures::A3135FirmwareVersion,
    pub serial_number: SerialNumber,
}

impl FromPacketBody for A3135StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3135 state update packet",
            map(
                (
                    a3135::structures::Volume::take,                // [0] volume
                    BatteryLevel::take,                             // [1] battery
                    le_u8,                                          // [2] unknown
                    le_u8,                                          // [3] unknown
                    VoicePrompt::take,                              // [4] voice prompt
                    le_u8,                                          // [5] unknown
                    le_u8,                                          // [6] unknown
                    a3135::structures::A3135FirmwareVersion::take,  // [7..12]
                    SerialNumber::take,                             // [12..28]
                    le_u8,                                          // [28] unknown
                ),
                |(volume, battery_level, _, _, voice_prompt, _, _, firmware_version, serial_number, _)| {
                    Self {
                        volume,
                        battery_level,
                        voice_prompt,
                        firmware_version,
                        serial_number,
                    }
                },
            ),
        )
        .parse_complete(input)
    }
}

impl ToPacket for A3135StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn command(&self) -> Command {
        packet::inbound::STATE_COMMAND
    }

    fn body(&self) -> Vec<u8> {
        self.volume
            .bytes()
            .chain(iter::once(self.battery_level.0))
            .chain([0x00, 0x01])                          // [2..4] unknown
            .chain(self.voice_prompt.bytes())             // [4]
            .chain([0x00, 0x03])                          // [5..7] unknown
            .chain(self.firmware_version.bytes())
            .chain(self.serial_number.as_str().as_bytes().iter().copied())
            .chain(iter::once(0u8))
            .collect()
    }
}

struct StateUpdatePacketHandler;

#[async_trait]
impl PacketHandler<A3135State> for StateUpdatePacketHandler {
    async fn handle_packet(
        &self,
        state: &watch::Sender<A3135State>,
        packet: &packet::Inbound,
    ) -> device::Result<()> {
        let packet: A3135StateUpdatePacket = packet.try_to_packet()?;
        state.send_modify(|state| state.update(packet));
        Ok(())
    }
}

impl ModuleCollection<A3135State> {
    pub fn add_state_update(&mut self) {
        self.packet_handlers.set_handler(
            packet::inbound::STATE_COMMAND,
            Box::new(StateUpdatePacketHandler {}),
        );
    }
}

/// CMD [10 93] response: 1 byte (current brightness level)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct A3135BrightnessPacket {
    pub brightness: a3135::structures::LedBrightness,
}

impl FromPacketBody for A3135BrightnessPacket {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "A3135BrightnessPacket",
            map(a3135::structures::LedBrightness::take, |brightness| Self {
                brightness,
            }),
        )
        .parse_complete(input)
    }
}

impl ToPacket for A3135BrightnessPacket {
    type DirectionMarker = packet::InboundMarker;

    fn command(&self) -> Command {
        REQUEST_BRIGHTNESS_COMMAND
    }

    fn body(&self) -> Vec<u8> {
        self.brightness.bytes().collect()
    }
}

/// CMD [01 7F] response: 1 byte [ldac_state]
/// 0x00=Combine, 0x01=LDAC active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct A3135LdacStatePacket {
    pub ldac: Ldac,
}

impl FromPacketBody for A3135LdacStatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "A3135LdacStatePacket",
            map(Ldac::take, |ldac| Self { ldac }),
        )
        .parse_complete(input)
    }
}

impl ToPacket for A3135LdacStatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn command(&self) -> Command {
        REQUEST_LDAC_STATE_COMMAND
    }

    fn body(&self) -> Vec<u8> {
        vec![self.ldac.bytes()[0]]
    }
}

pub const REQUEST_EQ_COMMAND: packet::Command = packet::Command([0x02, 0x89]);

/// CMD [02 89] response: 57 bytes
/// [0] unknown [1] unknown [2] active_preset_index
/// [3..21] Custom profile 1: 9 pairs of [amp_byte, freq_byte]
/// [21..57] Custom profiles 2 and 3 (ignored)
///
/// Amplitude encoding: device byte 0x3C=min(-6dB), 0x78=0dB, 0xB4=max(+6dB)
/// adj (tenths of dB) = device_byte - 120
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct A3135EqPacket {
    pub equalizer_configuration: EqualizerConfiguration<1, 9, -60, 60, 1>,
}

impl FromPacketBody for A3135EqPacket {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "A3135EqPacket",
            map(take(57usize), |bytes: &[u8]| {
                let active_preset = bytes[2] as u16;
                // bytes[3..21]: 9 pairs of [amp_byte, freq_byte] for custom profile 1
                let adj: [i16; 9] = array::from_fn(|i| bytes[3 + i * 2] as i16 - 120);
                let volume_adjustments = VolumeAdjustments::new(adj);
                Self {
                    equalizer_configuration: EqualizerConfiguration::new(
                        active_preset,
                        [volume_adjustments],
                    ),
                }
            }),
        )
        .parse_complete(input)
    }
}

impl ToPacket for A3135EqPacket {
    type DirectionMarker = packet::InboundMarker;

    fn command(&self) -> Command {
        REQUEST_EQ_COMMAND
    }

    fn body(&self) -> Vec<u8> {
        // Default response: active preset = 0 (Balanced), all bands at 0dB (0x78)
        const FREQ_CODES: [u8; 9] = [0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x06, 0x01];
        let preset = self.equalizer_configuration.preset_id() as u8;
        let amps = self.equalizer_configuration.volume_adjustments_channel_1().bytes();
        let mut body = vec![0x00, 0x00, preset];
        for i in 0..9 {
            body.push(amps[i] + 0x3C); // convert: adj+60+60 = adj+120 = device byte
            body.push(FREQ_CODES[i]);
        }
        // Two more empty profiles (36 bytes)
        body.extend(std::iter::repeat(0x78u8).take(9).flat_map(|a| [a, 0x07u8]));
        body.extend(std::iter::repeat(0x78u8).take(9).flat_map(|a| [a, 0x07u8]));
        body
    }
}

#[cfg(test)]
mod tests {
    use nom_language::error::VerboseError;

    use crate::devices::soundcore::common::packet::inbound::TryToPacket;

    use super::*;

    #[test]
    fn serialize_and_deserialize() {
        let bytes = A3135StateUpdatePacket::default()
            .to_packet()
            .bytes_with_checksum();
        let (_, packet) = packet::Inbound::take_with_checksum::<VerboseError<_>>(&bytes).unwrap();
        let _: A3135StateUpdatePacket = packet.try_to_packet().unwrap();
    }

    #[test]
    fn parse_real_packet() {
        // Real capture: 09 FF 00 00 01 01 01 27 00 [29-byte body] 7E
        let body: &[u8] = &[
            0x0E, 0x05, 0x01, 0x01, 0x01, 0x00, 0x03, 0x34, 0x2E, 0x30, 0x2E, 0x34, 0x41, 0x43,
            0x43, 0x4C, 0x56, 0x48, 0x32, 0x46, 0x33, 0x34, 0x32, 0x30, 0x32, 0x38, 0x39, 0x33,
            0x77,
        ];
        let (_, packet) =
            A3135StateUpdatePacket::take::<VerboseError<_>>(body).expect("parse failed");
        assert_eq!(packet.battery_level.0, 5);
        assert_eq!(packet.firmware_version.to_string(), "4.0.4");
        assert_eq!(packet.serial_number.as_str(), "ACCLVH2F34202893");
    }
}
