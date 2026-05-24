use std::collections::HashMap;

use crate::devices::soundcore::{
    a3961::{packets::inbound::A3961StateUpdatePacket, state::A3961State},
    common::{
        device::fetch_state_from_state_update_packet,
        macros::soundcore_device,
        modules::equalizer,
        packet::outbound::{RequestState, ToPacket},
    },
};

mod modules;
mod packets;
mod state;
mod structures;

soundcore_device!(
    A3961State,
    async |packet_io| {
        fetch_state_from_state_update_packet::<A3961State, A3961StateUpdatePacket>(packet_io).await
    },
    async |builder| {
        builder.module_collection().add_state_update();
        builder.a3961_sound_modes();
        builder.equalizer_tws(equalizer::common_settings()).await;
        builder.tws_status();
        builder.dual_battery(5);
        builder.serial_number_and_dual_firmware_version();
    },
    {
        HashMap::from([(
            RequestState::COMMAND,
            A3961StateUpdatePacket::default().to_packet(),
        )])
    },
);
