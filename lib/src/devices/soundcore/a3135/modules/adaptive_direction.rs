use std::sync::Arc;

use openscq30_lib_has::Has;
use strum::{EnumIter, EnumString};

use crate::{
    api::settings::{CategoryId, SettingId},
    devices::soundcore::{
        a3135,
        common::{modules::ModuleCollection, packet, packet::PacketIOController},
    },
    macros::enum_subset,
};

mod packet_handler;
mod setting_handler;
mod state_modifier;

enum_subset!(
    SettingId,
    #[derive(EnumIter, EnumString)]
    enum AdaptiveDirectionSetting {
        AdaptiveDirection,
    }
);

impl<T> ModuleCollection<T>
where
    T: Has<a3135::structures::AdaptiveDirection> + Clone + Send + Sync + 'static,
{
    pub fn add_a3135_adaptive_direction(&mut self, packet_io: Arc<PacketIOController>) {
        self.setting_manager.add_handler(
            CategoryId::Miscellaneous,
            setting_handler::AdaptiveDirectionSettingHandler,
        );
        self.state_modifiers
            .push(Box::new(state_modifier::AdaptiveDirectionStateModifier::new(
                packet_io,
            )));
        self.packet_handlers.set_handler(
            packet::Command([0x02, 0x8C]),
            Box::new(packet_handler::AdaptiveDirectionPacketHandler),
        );
    }
}
