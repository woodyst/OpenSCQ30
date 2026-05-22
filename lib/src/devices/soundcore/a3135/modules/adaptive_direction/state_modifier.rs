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

pub struct AdaptiveDirectionStateModifier {
    packet_io: Arc<PacketIOController>,
}

impl AdaptiveDirectionStateModifier {
    pub fn new(packet_io: Arc<PacketIOController>) -> Self {
        Self { packet_io }
    }
}

#[async_trait]
impl<T> StateModifier<T> for AdaptiveDirectionStateModifier
where
    T: Has<a3135::structures::AdaptiveDirection> + Clone + Send + Sync,
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
        // Two-packet SET: CMD [02 8A] (no response) then CMD [02 8C] (waits for response)
        self.packet_io
            .send_without_response(&a3135::packets::outbound::set_adaptive_direction(target.0))
            .await?;
        self.packet_io
            .send_with_response(&a3135::packets::outbound::confirm_adaptive_direction())
            .await?;
        state_sender.send_modify(|state| {
            *state.get_mut() = target;
        });
        Ok(())
    }
}
