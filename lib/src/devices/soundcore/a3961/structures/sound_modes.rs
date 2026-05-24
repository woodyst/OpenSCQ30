use nom::{
    IResult, Parser,
    combinator::map,
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};
use openscq30_i18n_macros::Translate;
use strum::{Display, EnumIter, EnumString, FromRepr, IntoStaticStr, VariantArray};

use crate::devices::soundcore::common::{
    modules::sound_modes_v2::ToPacketBody,
    packet::{self, inbound::FromPacketBody},
};

#[repr(u8)]
#[derive(
    FromRepr,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Display,
    Default,
    IntoStaticStr,
    EnumString,
    EnumIter,
    VariantArray,
    Translate,
)]
pub enum A3961AmbientSoundMode {
    #[default]
    NoiseCanceling = 0,
    Transparency = 1,
    Normal = 2,
}

impl A3961AmbientSoundMode {
    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3961 ambient sound mode",
            map(le_u8, |id| Self::from_repr(id).unwrap_or_default()),
        )
        .parse_complete(input)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct A3961SoundModes {
    pub ambient_sound_mode: A3961AmbientSoundMode,
}

impl FromPacketBody for A3961SoundModes {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3961 sound modes",
            map(A3961AmbientSoundMode::take, |ambient_sound_mode| Self {
                ambient_sound_mode,
            }),
        )
        .parse_complete(input)
    }
}

impl ToPacketBody for A3961SoundModes {
    fn bytes(&self) -> Vec<u8> {
        vec![self.ambient_sound_mode as u8]
    }
}
