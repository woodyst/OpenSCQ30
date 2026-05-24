use std::{fmt::Display, iter};

use nom::{
    IResult, Parser,
    bytes::complete::take,
    combinator::map,
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};
use openscq30_i18n::Translate;
use strum::{EnumIter, IntoStaticStr};

use crate::i18n::fl;

/// Firmware version for A3135, stored as raw ASCII bytes in "X.Y.Z" format.
/// The standard FirmwareVersion only handles "XX.YY"; A3135 uses a three-part version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct A3135FirmwareVersion {
    bytes: [u8; 5],
}

impl Default for A3135FirmwareVersion {
    fn default() -> Self {
        Self { bytes: *b"0.0.0" }
    }
}

impl A3135FirmwareVersion {
    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "a3135 firmware version",
            map(take(5usize), |bytes: &[u8]| {
                let mut arr = [0u8; 5];
                arr.copy_from_slice(bytes);
                Self { bytes: arr }
            }),
        )
        .parse_complete(input)
    }

    pub fn bytes(&self) -> [u8; 5] {
        self.bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Volume(pub u8);

impl Volume {
    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context("volume", map(le_u8, Self)).parse_complete(input)
    }

    pub fn bytes(&self) -> impl Iterator<Item = u8> {
        iter::once(self.0)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PowerOffPending(pub bool);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumIter, IntoStaticStr)]
pub enum AdaptiveDirection {
    #[default]
    #[strum(serialize = "standing")]
    Standing,
    #[strum(serialize = "horizontal")]
    Horizontal,
    #[strum(serialize = "hanging")]
    Hanging,
}

impl AdaptiveDirection {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x01 => Self::Horizontal,
            0x02 => Self::Hanging,
            _ => Self::Standing,
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            Self::Standing => 0x00,
            Self::Horizontal => 0x01,
            Self::Hanging => 0x02,
        }
    }
}

impl Translate for AdaptiveDirection {
    fn translate(&self) -> String {
        match self {
            Self::Standing => fl!("standing"),
            Self::Horizontal => fl!("horizontal"),
            Self::Hanging => fl!("hanging"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, EnumIter, IntoStaticStr)]
pub enum LedBrightness {
    #[default]
    #[strum(serialize = "off")]
    Off,
    #[strum(serialize = "low")]
    Low,
    #[strum(serialize = "medium")]
    Medium,
    #[strum(serialize = "high")]
    High,
}

impl LedBrightness {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x14 => Self::Low,
            0x46 => Self::Medium,
            0x64 => Self::High,
            _ => Self::Off,
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            Self::Off => 0x00,
            Self::Low => 0x14,
            Self::Medium => 0x46,
            Self::High => 0x64,
        }
    }

    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context("led brightness", map(le_u8, Self::from_byte)).parse_complete(input)
    }

    pub fn bytes(self) -> impl Iterator<Item = u8> {
        iter::once(self.to_byte())
    }
}

impl Translate for LedBrightness {
    fn translate(&self) -> String {
        match self {
            Self::Off => fl!("off"),
            Self::Low => fl!("low"),
            Self::Medium => fl!("medium"),
            Self::High => fl!("high"),
        }
    }
}

impl Display for A3135FirmwareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(&self.bytes) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "{}", self.bytes.map(|b| format!("{b:02X}")).join(" ")),
        }
    }
}
