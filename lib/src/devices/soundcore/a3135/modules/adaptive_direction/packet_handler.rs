use async_trait::async_trait;
use openscq30_lib_has::Has;
use tokio::sync::watch;

use crate::{
    api::device,
    devices::soundcore::{
        a3135,
        common::{packet, packet_manager::PacketHandler},
    },
};

pub struct AdaptiveDirectionPacketHandler;

#[async_trait]
impl<T> PacketHandler<T> for AdaptiveDirectionPacketHandler
where
    T: Has<a3135::structures::AdaptiveDirection> + Send + Sync,
{
    async fn handle_packet(
        &self,
        state: &watch::Sender<T>,
        packet: &packet::Inbound,
    ) -> device::Result<()> {
        if let Some(byte) = packet.body.first() {
            let direction = a3135::structures::AdaptiveDirection::from_byte(*byte);
            state.send_if_modified(|s| {
                let current = s.get_mut();
                let changed = *current != direction;
                *current = direction;
                changed
            });
        }
        Ok(())
    }
}
