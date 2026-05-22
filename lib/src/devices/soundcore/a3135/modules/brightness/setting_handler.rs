use async_trait::async_trait;
use openscq30_lib_has::Has;
use strum::IntoEnumIterator;

use crate::{
    api::settings::{Select, Setting, SettingId, Value, ValueError},
    devices::soundcore::{
        a3135::{self, modules::brightness::BrightnessSetting},
        common::settings_manager::{SettingHandler, SettingHandlerResult},
    },
};

pub struct BrightnessSettingHandler;

#[async_trait]
impl<T> SettingHandler<T> for BrightnessSettingHandler
where
    T: Has<a3135::structures::LedBrightness> + Send,
{
    fn settings(&self) -> Vec<SettingId> {
        BrightnessSetting::iter().map(Into::into).collect()
    }

    fn get(&self, state: &T, setting_id: &SettingId) -> Option<Setting> {
        let brightness = state.get();
        let _: BrightnessSetting = (*setting_id).try_into().ok()?;
        Some(Setting::Select {
            setting: Select::from_enum(a3135::structures::LedBrightness::iter()),
            value: <&'static str>::from(brightness).into(),
        })
    }

    async fn set(
        &self,
        state: &mut T,
        setting_id: &SettingId,
        value: Value,
    ) -> SettingHandlerResult<()> {
        let _: BrightnessSetting = (*setting_id)
            .try_into()
            .expect("already filtered to valid values only by SettingsManager");
        let selection = value.try_as_str()?;
        let brightness = a3135::structures::LedBrightness::iter()
            .find(|level| <&'static str>::from(level).eq_ignore_ascii_case(selection))
            .ok_or_else(|| ValueError::InvalidEnumVariant {
                variants: a3135::structures::LedBrightness::iter()
                    .map(<&'static str>::from)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                actual: value.clone(),
            })?;
        *state.get_mut() = brightness;
        Ok(())
    }
}
