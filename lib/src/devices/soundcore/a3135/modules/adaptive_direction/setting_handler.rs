use async_trait::async_trait;
use openscq30_lib_has::Has;
use strum::IntoEnumIterator;

use crate::{
    api::settings::{Select, Setting, SettingId, Value, ValueError},
    devices::soundcore::{
        a3135::{self, modules::adaptive_direction::AdaptiveDirectionSetting},
        common::settings_manager::{SettingHandler, SettingHandlerResult},
    },
};

pub struct AdaptiveDirectionSettingHandler;

#[async_trait]
impl<T> SettingHandler<T> for AdaptiveDirectionSettingHandler
where
    T: Has<a3135::structures::AdaptiveDirection> + Send,
{
    fn settings(&self) -> Vec<SettingId> {
        AdaptiveDirectionSetting::iter().map(Into::into).collect()
    }

    fn get(&self, state: &T, setting_id: &SettingId) -> Option<Setting> {
        let dir = state.get();
        let _: AdaptiveDirectionSetting = (*setting_id).try_into().ok()?;
        Some(Setting::Select {
            setting: Select::from_enum(a3135::structures::AdaptiveDirection::iter()),
            value: <&'static str>::from(dir).into(),
        })
    }

    async fn set(
        &self,
        state: &mut T,
        setting_id: &SettingId,
        value: Value,
    ) -> SettingHandlerResult<()> {
        let _: AdaptiveDirectionSetting = (*setting_id)
            .try_into()
            .expect("already filtered to valid values only by SettingsManager");
        let selection = value.try_as_str()?;
        let direction = a3135::structures::AdaptiveDirection::iter()
            .find(|d| <&'static str>::from(d).eq_ignore_ascii_case(selection))
            .ok_or_else(|| ValueError::InvalidEnumVariant {
                variants: a3135::structures::AdaptiveDirection::iter()
                    .map(<&'static str>::from)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                actual: value.clone(),
            })?;
        *state.get_mut() = direction;
        Ok(())
    }
}
