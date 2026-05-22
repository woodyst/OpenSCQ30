use std::sync::Arc;

use openscq30_lib_has::MaybeHas;
use strum::{EnumIter, EnumString};

use crate::{
    api::settings::{CategoryId, SettingId},
    devices::soundcore::{
        a3135,
        common::{modules::ModuleCollection, packet::PacketIOController},
    },
    macros::enum_subset,
};

mod setting_handler;
mod state_modifier;

enum_subset!(
    SettingId,
    #[derive(EnumIter, EnumString)]
    enum VolumeSetting {
        Volume,
    }
);

impl<T> ModuleCollection<T>
where
    T: MaybeHas<a3135::structures::Volume> + Clone + Send + Sync + 'static,
{
    pub fn add_a3135_volume(&mut self, packet_io: Arc<PacketIOController>, max_volume: u8) {
        self.setting_manager.add_handler(
            CategoryId::Miscellaneous,
            setting_handler::VolumeSettingHandler::new(max_volume),
        );
        self.state_modifiers
            .push(Box::new(state_modifier::VolumeStateModifier::new(
                packet_io,
            )));
    }
}
