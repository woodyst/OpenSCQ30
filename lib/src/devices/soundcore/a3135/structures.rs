use std::{fmt::Display, iter};

use nom::{
    IResult, Parser,
    bytes::complete::take,
    combinator::map,
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};

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

impl Display for A3135FirmwareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match std::str::from_utf8(&self.bytes) {
            Ok(s) => write!(f, "{s}"),
            Err(_) => write!(f, "{}", self.bytes.map(|b| format!("{b:02X}")).join(" ")),
        }
    }
}
