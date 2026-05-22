use async_trait::async_trait;
use openscq30_lib_has::Has;
use std::sync::Arc;
use tokio::sync::watch;

use crate::{
    api::device,
    devices::soundcore::{
        a3135,
        common::{packet::PacketIOController, state_modifier::StateModifier},
    },
};

pub struct BrightnessStateModifier {
    packet_io: Arc<PacketIOController>,
}

impl BrightnessStateModifier {
    pub fn new(packet_io: Arc<PacketIOController>) -> Self {
        Self { packet_io }
    }
}

#[async_trait]
impl<T> StateModifier<T> for BrightnessStateModifier
where
    T: Has<a3135::structures::LedBrightness> + Clone + Send + Sync,
{
    async fn move_to_state(
        &self,
        state_sender: &watch::Sender<T>,
        target_state: &T,
    ) -> device::Result<()> {
        let target = *target_state.get();
        {
            let current = state_sender.borrow();
            if *current.get() == target {
                return Ok(());
            }
        }
        self.packet_io
            .send_with_response(&a3135::packets::outbound::set_brightness(target))
            .await?;
        state_sender.send_modify(|state| {
            *state.get_mut() = target;
        });
        Ok(())
    }
}
