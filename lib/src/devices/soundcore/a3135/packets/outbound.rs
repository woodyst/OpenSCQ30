use crate::devices::soundcore::{a3135, common::{packet, structures::VolumeAdjustments}};

pub fn set_volume(volume: &a3135::structures::Volume) -> packet::Outbound {
    packet::Outbound::new(packet::Command([0x01, 0x88]), volume.bytes().collect())
}

pub fn power_off() -> packet::Outbound {
    packet::Outbound::new(packet::Command([0x01, 0x89]), Vec::new())
}

pub fn set_brightness(brightness: a3135::structures::LedBrightness) -> packet::Outbound {
    packet::Outbound::new(packet::Command([0x10, 0x92]), brightness.bytes().collect())
}


pub fn set_eq_preset(preset_id: u8) -> packet::Outbound {
    packet::Outbound::new(packet::Command([0x02, 0x8B]), vec![preset_id])
}

/// SET custom EQ — CMD [02 8D], 21-byte body:
/// [0x07, 0x01, 0xFF] + 9 pairs of [amp_byte, freq_byte]
/// amp_byte = adj + 120 (device range 0x3C–0xB4, 0x78=0dB)
pub fn set_custom_eq(volume_adjustments: VolumeAdjustments<9, -60, 60, 1>) -> packet::Outbound {
    const FREQ_CODES: [u8; 9] = [0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x06, 0x01];
    let amps = volume_adjustments.bytes(); // adj+60; add 0x3C to get device byte
    let mut body = vec![0x07u8, 0x01, 0xFF];
    for i in 0..9 {
        body.push(amps[i] + 0x3C);
        body.push(FREQ_CODES[i]);
    }
    packet::Outbound::new(packet::Command([0x02, 0x8D]), body)
}
