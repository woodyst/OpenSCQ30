use std::iter;

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
            structures::{BatteryLevel, Ldac, SerialNumber},
        },
    },
};

pub const REQUEST_BRIGHTNESS_COMMAND: packet::Command = packet::Command([0x10, 0x93]);

/// Body layout (29 bytes):
///   [0]       volume (0–31)
///   [1]       battery level (0–5)
///   [2..7]    unknown (5 bytes)
///   [7..12]   firmware version ASCII "X.Y.Z" (5 bytes)
///   [12..28]  serial number ASCII (16 bytes)
///   [28]      unknown (1 byte)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct A3135StateUpdatePacket {
    pub volume: a3135::structures::Volume,
    pub battery_level: BatteryLevel,
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
                    take(5usize),                                   // [2..7] unknown
                    a3135::structures::A3135FirmwareVersion::take,  // [7..12]
                    SerialNumber::take,                             // [12..28]
                    le_u8,                                          // [28] unknown
                ),
                |(volume, battery_level, _unknown, firmware_version, serial_number, _unknown2)| {
                    Self {
                        volume,
                        battery_level,
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
            .chain([0u8; 5]) // unknown [2..7]
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

/// CMD [01 7F] response: 2 bytes [ldac_state, unknown]
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
            map(
                (Ldac::take, le_u8), // [0] ldac state, [1] unknown extra byte
                |(ldac, _unknown)| Self { ldac },
            ),
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
        vec![self.ldac.bytes()[0], 0x00]
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
