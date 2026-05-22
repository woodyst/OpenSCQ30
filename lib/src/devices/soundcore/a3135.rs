use std::collections::HashMap;

use uuid::uuid;

use crate::connection::RfcommServiceSelectionStrategy;
use crate::devices::soundcore::a3135::packets::inbound::{
    A3135BrightnessPacket, A3135EqPacket, A3135LdacStatePacket, A3135StateUpdatePacket,
    REQUEST_BRIGHTNESS_COMMAND, REQUEST_EQ_COMMAND,
};
use crate::devices::soundcore::a3135::state::A3135State;
use crate::devices::soundcore::common::device::SoundcoreDeviceConfig;
use crate::devices::soundcore::common::macros::soundcore_device;
use crate::devices::soundcore::common::modules::auto_power_off::AutoPowerOffDuration;
use crate::devices::soundcore::common::packet::{self, inbound::TryToPacket, outbound::{RequestState, ToPacket, REQUEST_LDAC_STATE_COMMAND}};

mod modules;
mod packets;
mod state;
mod structures;

soundcore_device!(
    A3135State,
    async |packet_io| {
        let state_update_packet: A3135StateUpdatePacket = packet_io
            .send_with_response(&RequestState.to_packet())
            .await?
            .try_to_packet()?;
        let ldac_packet: A3135LdacStatePacket = packet_io
            .send_with_response(&packet::Outbound::new(REQUEST_LDAC_STATE_COMMAND, Vec::new()))
            .await?
            .try_to_packet()?;
        let brightness_packet: A3135BrightnessPacket = packet_io
            .send_with_response(&packet::Outbound::new(REQUEST_BRIGHTNESS_COMMAND, Vec::new()))
            .await?
            .try_to_packet()?;
        let eq_packet: A3135EqPacket = packet_io
            .send_with_response(&packet::Outbound::new(REQUEST_EQ_COMMAND, Vec::new()))
            .await?
            .try_to_packet()?;
        Ok(A3135State::new(
            state_update_packet,
            ldac_packet.ldac,
            brightness_packet.brightness,
            eq_packet,
        ))
    },
    async |builder| {
        builder.module_collection().add_state_update();
        builder.single_battery_level(5);
        builder.a3135_serial_number_and_firmware_version();
        builder.a3135_volume(31);
        builder.ldac();
        builder.voice_prompt();
        builder.auto_power_off(AutoPowerOffDuration::five_ten_twenty_sixty());
        builder.a3135_power_off();
        builder.a3135_brightness();
        builder.a3135_adaptive_direction();
        builder.a3135_equalizer().await;
    },
    {
        HashMap::from([
            (
                RequestState::COMMAND,
                A3135StateUpdatePacket::default().to_packet(),
            ),
            (
                REQUEST_LDAC_STATE_COMMAND,
                A3135LdacStatePacket::default().to_packet(),
            ),
            (
                REQUEST_BRIGHTNESS_COMMAND,
                A3135BrightnessPacket::default().to_packet(),
            ),
            (
                REQUEST_EQ_COMMAND,
                A3135EqPacket::default().to_packet(),
            ),
        ])
    },
    CONFIG,
);

const CONFIG: SoundcoreDeviceConfig = SoundcoreDeviceConfig {
    checksum_kind: packet::ChecksumKind::None,
    rfcomm_service_selection_strategy: RfcommServiceSelectionStrategy::Constant(uuid!(
        "0cf12d31-fac3-4553-bd80-d6832e7b3135"
    )),
};

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::{
        DeviceModel,
        devices::soundcore::common::{
            device::test_utils::TestSoundcoreDevice,
            packet,
        },
        settings::SettingId,
    };

    use super::*;

    #[tokio::test(start_paused = true)]
    async fn parse_real_state_packet() {
        let device = TestSoundcoreDevice::new(
            super::device_registry,
            DeviceModel::SoundcoreA3135,
            HashMap::from([
                (
                    packet::Command([1, 1]),
                    packet::Inbound::new(
                        packet::Command([1, 1]),
                        vec![
                            0x0E, 0x05, 0x01, 0x01, 0x01, 0x00, 0x03, 0x34, 0x2E, 0x30, 0x2E,
                            0x34, 0x41, 0x43, 0x43, 0x4C, 0x56, 0x48, 0x32, 0x46, 0x33, 0x34,
                            0x32, 0x30, 0x32, 0x38, 0x39, 0x33, 0x77,
                        ],
                    ),
                ),
                (
                    REQUEST_LDAC_STATE_COMMAND,
                    packet::Inbound::new(
                        REQUEST_LDAC_STATE_COMMAND,
                        vec![0x01, 0x00], // 0x01=LDAC active, 0x00=unknown extra byte
                    ),
                ),
                (
                    REQUEST_BRIGHTNESS_COMMAND,
                    packet::Inbound::new(
                        REQUEST_BRIGHTNESS_COMMAND,
                        vec![0x46], // 0x46=Medium
                    ),
                ),
                (
                    REQUEST_EQ_COMMAND,
                    packet::Inbound::new(
                        REQUEST_EQ_COMMAND,
                        // 57 bytes: [0x00, 0x00, active_preset=0x00] + 3 profiles × 18 bytes
                        // Profile 1: 9 pairs [0x78(0dB), freq_code]
                        {
                            let mut v = vec![0x00u8, 0x00, 0x00]; // header + active=Balanced
                            let freq_codes: [u8; 9] = [0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x06, 0x01];
                            for f in freq_codes { v.push(0x78); v.push(f); } // profile 1 (18 bytes)
                            v.extend_from_slice(&[0x78u8; 18]); // profile 2 (ignored)
                            v.extend_from_slice(&[0x78u8; 18]); // profile 3 (ignored)
                            v
                        },
                    ),
                ),
            ]),
            CONFIG,
        )
        .await;

        device.assert_setting_values([
            (SettingId::BatteryLevel, Cow::from("5/5").into()),
            (SettingId::FirmwareVersion, Cow::from("4.0.4").into()),
            (
                SettingId::SerialNumber,
                Cow::from("ACCLVH2F34202893").into(),
            ),
            (SettingId::Volume, 14i32.into()),
            (SettingId::Ldac, true.into()),
            (SettingId::LedBrightness, Cow::from("medium").into()),
        ]);
    }
}
