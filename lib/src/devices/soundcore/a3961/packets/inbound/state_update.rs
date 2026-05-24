use async_trait::async_trait;
use nom::{
    IResult, Parser,
    bytes::complete::take,
    combinator::map,
    error::{ContextError, ParseError, context},
};
use tokio::sync::watch;

use crate::{
    device,
    devices::soundcore::{
        a3961::{
            state::A3961State,
            structures::{A3961AmbientSoundMode, A3961SoundModes},
        },
        common::{
            modules::ModuleCollection,
            packet::{
                self, Command,
                inbound::{FromPacketBody, TryToPacket},
                outbound::ToPacket,
            },
            packet_manager::PacketHandler,
            structures::{
                CommonEqualizerConfiguration, DualBattery, DualFirmwareVersion, SerialNumber,
                TwsStatus,
            },
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A3961StateUpdatePacket {
    pub tws_status: TwsStatus,
    pub battery: DualBattery,
    pub firmware_version: DualFirmwareVersion,
    pub serial_number: SerialNumber,
    pub equalizer_configuration: CommonEqualizerConfiguration<1, 10>,
    pub sound_modes: A3961SoundModes,
}

impl Default for A3961StateUpdatePacket {
    fn default() -> Self {
        Self {
            tws_status: Default::default(),
            battery: Default::default(),
            firmware_version: Default::default(),
            serial_number: Default::default(),
            equalizer_configuration: Default::default(),
            sound_modes: Default::default(),
        }
    }
}

impl FromPacketBody for A3961StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3961 state update packet",
            map(
                (
                    TwsStatus::take,
                    DualBattery::take,
                    DualFirmwareVersion::take,
                    SerialNumber::take,
                    CommonEqualizerConfiguration::<1, 10>::take, // bytes [32-43]
                    take(72usize),                               // bytes [44-115] unknown
                    A3961AmbientSoundMode::take,                 // byte [116]
                ),
                |(
                    tws_status,
                    battery,
                    firmware_version,
                    serial_number,
                    equalizer_configuration,
                    _,
                    ambient_sound_mode,
                )| Self {
                    tws_status,
                    battery,
                    firmware_version,
                    serial_number,
                    equalizer_configuration,
                    sound_modes: A3961SoundModes { ambient_sound_mode },
                },
            ),
        )
        .parse(input)
    }
}

impl ToPacket for A3961StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn command(&self) -> Command {
        packet::inbound::STATE_COMMAND
    }

    fn body(&self) -> Vec<u8> {
        self.tws_status
            .bytes()
            .into_iter()
            .chain(self.battery.bytes())
            .chain(self.firmware_version.bytes())
            .chain(self.serial_number.bytes())
            .chain(self.equalizer_configuration.bytes()) // bytes [32-43]
            .chain([0u8; 72]) // bytes [44-115]
            .chain([self.sound_modes.ambient_sound_mode as u8]) // byte [116]
            .collect()
    }
}

struct StateUpdatePacketHandler;

#[async_trait]
impl PacketHandler<A3961State> for StateUpdatePacketHandler {
    async fn handle_packet(
        &self,
        state: &watch::Sender<A3961State>,
        packet: &packet::Inbound,
    ) -> device::Result<()> {
        let packet: A3961StateUpdatePacket = packet.try_to_packet()?;
        state.send_modify(|state| *state = packet.into());
        Ok(())
    }
}

impl ModuleCollection<A3961State> {
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
        let bytes = A3961StateUpdatePacket::default()
            .to_packet()
            .bytes_with_checksum();
        let (_, packet) = packet::Inbound::take_with_checksum::<VerboseError<_>>(&bytes).unwrap();
        let _: A3961StateUpdatePacket = packet.try_to_packet().unwrap();
    }
}
