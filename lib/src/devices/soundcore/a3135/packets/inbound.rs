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
                outbound::ToPacket,
            },
            packet_manager::PacketHandler,
            state::Update,
            structures::{BatteryLevel, SerialNumber},
        },
    },
};

/// Body layout (29 bytes):
///   [0]       volume (0–?)
///   [1]       battery level (0–5)
///   [2..7]    unknown (5 bytes)
///   [7..12]   firmware version ASCII "X.Y.Z" (5 bytes)
///   [12..28]  serial number ASCII (16 bytes)
///   [28]      unknown (1 byte)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct A3135StateUpdatePacket {
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
                    le_u8, // volume
                    BatteryLevel::take,
                    take(5usize),
                    a3135::structures::A3135FirmwareVersion::take,
                    SerialNumber::take,
                    le_u8,
                ),
                |(_volume, battery_level, _unknown1, firmware_version, serial_number, _unknown2)| {
                    Self {
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
        iter::once(0u8) // volume (unknown)
            .chain(iter::once(self.battery_level.0))
            .chain([0u8; 5])
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
