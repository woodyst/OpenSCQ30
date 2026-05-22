use crate::devices::soundcore::{a3135, common::packet};

pub fn set_volume(volume: &a3135::structures::Volume) -> packet::Outbound {
    packet::Outbound::new(packet::Command([0x01, 0x88]), volume.bytes().collect())
}

pub fn power_off() -> packet::Outbound {
    packet::Outbound::new(packet::Command([0x01, 0x89]), Vec::new())
}
