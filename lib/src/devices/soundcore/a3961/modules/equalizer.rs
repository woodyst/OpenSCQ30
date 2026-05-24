use crate::{
    devices::soundcore::common::{
        modules::equalizer::{self, EqualizerModuleSettings},
        structures::VolumeAdjustments,
    },
    i18n::fl,
};

pub fn a3961_equalizer_settings() -> EqualizerModuleSettings<8, 8, -120, 134, 1> {
    let mut settings = equalizer::common_settings();
    if let Some(preset) = settings.presets.iter_mut().find(|p| p.id == 0x0002) {
        preset.name = "BassUp";
        preset.localized_name = || fl!("bass-up");
        preset.volume_adjustments = VolumeAdjustments::new([50, 50, 30, 10, 10, 20, -10, -20]);
    }
    settings
}
