use std::sync::Arc;

use openscq30_lib_has::Has;
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
    enum BrightnessSetting {
        LedBrightness,
    }
);

impl<T> ModuleCollection<T>
where
    T: Has<a3135::structures::LedBrightness> + Clone + Send + Sync + 'static,
{
    pub fn add_a3135_brightness(&mut self, packet_io: Arc<PacketIOController>) {
        self.setting_manager.add_handler(
            CategoryId::Miscellaneous,
            setting_handler::BrightnessSettingHandler,
        );
        self.state_modifiers
            .push(Box::new(state_modifier::BrightnessStateModifier::new(
                packet_io,
            )));
    }
}
