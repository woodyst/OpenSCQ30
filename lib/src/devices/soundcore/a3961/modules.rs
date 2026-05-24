use openscq30_lib_has::Has;

use crate::devices::soundcore::common::device::SoundcoreDeviceBuilder;

use super::structures::A3961SoundModes;

mod equalizer;
mod sound_modes;

pub use equalizer::a3961_equalizer_settings;

impl<StateType> SoundcoreDeviceBuilder<StateType>
where
    StateType: Has<A3961SoundModes> + Send + Sync + Clone + 'static,
{
    pub fn a3961_sound_modes(&mut self) {
        let packet_io_controller = self.packet_io_controller().clone();
        self.module_collection()
            .add_a3961_sound_modes(packet_io_controller);
    }
}
