use async_trait::async_trait;
use openscq30_i18n::Translate;
use openscq30_lib_has::Has;
use strum::IntoEnumIterator;

use crate::{
    api::settings::{Setting, SettingId, Value},
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
        Some(Setting::Information {
            value: <&'static str>::from(dir).to_owned(),
            translated_value: dir.translate(),
        })
    }

    async fn set(
        &self,
        _state: &mut T,
        _setting_id: &SettingId,
        _value: Value,
    ) -> SettingHandlerResult<()> {
        Ok(())
    }
}
