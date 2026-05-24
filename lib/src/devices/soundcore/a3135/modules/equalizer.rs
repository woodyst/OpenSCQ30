use async_trait::async_trait;
use openscq30_lib_has::Has;
use std::sync::Arc;
use tokio::sync::watch;

use crate::{
    DeviceModel,
    api::device,
    devices::soundcore::{
        a3135::{self, packets::inbound::{A3135EqPacket, REQUEST_EQ_COMMAND}},
        common::{
            device::SoundcoreDeviceBuilder,
            modules::{
                ModuleCollection,
                equalizer::{EqualizerModuleSettings, EqualizerPreset},
            },
            packet::{self, PacketIOController, inbound::TryToPacket},
            state_modifier::StateModifier,
            structures::{EqualizerConfiguration, VolumeAdjustments},
        },
    },
    i18n::fl,
    storage::OpenSCQ30Database,
};

pub const CUSTOM_PRESET_ID: u16 = 0xFE;

struct EqualizerStateModifier {
    packet_io: Arc<PacketIOController>,
}

impl EqualizerStateModifier {
    fn new(packet_io: Arc<PacketIOController>) -> Self {
        Self { packet_io }
    }
}

#[async_trait]
impl<T> StateModifier<T> for EqualizerStateModifier
where
    T: Has<EqualizerConfiguration<1, 9, -60, 60, 1>> + Clone + Send + Sync,
{
    async fn move_to_state(
        &self,
        state_sender: &watch::Sender<T>,
        target_state: &T,
    ) -> device::Result<()> {
        let target: EqualizerConfiguration<1, 9, -60, 60, 1> = *target_state.get();
        let state: EqualizerConfiguration<1, 9, -60, 60, 1> = *state_sender.borrow().get();

        if target == state {
            return Ok(());
        }

        if target.preset_id() != state.preset_id() {
            self.packet_io
                .send_with_response(&a3135::packets::outbound::set_eq_preset(
                    target.preset_id() as u8,
                ))
                .await?;
            // Re-read EQ after preset change: hardware presets apply device-internal curves.
            let eq: A3135EqPacket = self.packet_io
                .send_with_response(&packet::Outbound::new(REQUEST_EQ_COMMAND, Vec::new()))
                .await?
                .try_to_packet()?;
            state_sender.send_modify(|s| *s.get_mut() = eq.equalizer_configuration);
            return Ok(());
        }

        if target.volume_adjustments() != state.volume_adjustments()
            && target.preset_id() == CUSTOM_PRESET_ID
        {
            self.packet_io
                .send_with_response(&a3135::packets::outbound::set_custom_eq(
                    *target.volume_adjustments_channel_1(),
                ))
                .await?;
        }

        state_sender.send_modify(|s| *s.get_mut() = target);
        Ok(())
    }
}

// PRESET_BANDS = 0: these are hardware presets — the device applies the curve internally.
// Selecting a preset only changes preset_id; band values in state are preserved (see setting_handler).
fn module_settings() -> EqualizerModuleSettings<9, 0, -60, 60, 1> {
    EqualizerModuleSettings {
        custom_preset_id: CUSTOM_PRESET_ID,
        band_hz: [80, 150, 300, 500, 1000, 2000, 4500, 8500, 15000],
        presets: vec![
            EqualizerPreset {
                name: "Balanced",
                localized_name: || fl!("balanced"),
                id: 0x00,
                volume_adjustments: VolumeAdjustments::new([]),
            },
            EqualizerPreset {
                name: "ExtraBass",
                localized_name: || fl!("extra-bass"),
                id: 0x01,
                volume_adjustments: VolumeAdjustments::new([]),
            },
            EqualizerPreset {
                name: "Voice",
                localized_name: || fl!("voice"),
                id: 0x02,
                volume_adjustments: VolumeAdjustments::new([]),
            },
            EqualizerPreset {
                name: "SoundcoreSignature",
                localized_name: || fl!("soundcore-signature"),
                id: 0x03,
                volume_adjustments: VolumeAdjustments::new([]),
            },
        ],
    }
}

impl<T> ModuleCollection<T>
where
    T: Has<EqualizerConfiguration<1, 9, -60, 60, 1>> + Clone + Send + Sync + 'static,
{
    pub async fn add_a3135_equalizer(
        &mut self,
        packet_io: Arc<PacketIOController>,
        database: Arc<OpenSCQ30Database>,
        device_model: DeviceModel,
        change_notify: watch::Sender<()>,
    ) {
        self.add_equalizer_with_custom_state_modifier::<1, 9, 9, 0, -60, 60, 1>(
            database,
            device_model,
            change_notify,
            Box::new(EqualizerStateModifier::new(packet_io)),
            module_settings(),
        )
        .await;
    }
}

impl<StateType> SoundcoreDeviceBuilder<StateType>
where
    StateType: Has<EqualizerConfiguration<1, 9, -60, 60, 1>> + Clone + Send + Sync + 'static,
{
    pub async fn a3135_equalizer(&mut self) {
        let packet_io = self.packet_io_controller().clone();
        let database = self.database();
        let device_model = self.device_model();
        let change_notify = self.change_notify();
        self.module_collection()
            .add_a3135_equalizer(packet_io, database, device_model, change_notify)
            .await;
    }
}
