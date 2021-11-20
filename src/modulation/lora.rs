//! Common LoRa modulation options

/// LoRa mode channel configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]

pub struct LoRaChannel {
    /// LoRa frequency in kHz
    pub freq_khz: u32,
    /// LoRa channel bandwidth
    pub bw_khz: u16,
    /// LoRa Spreading Factor
    pub sf: SpreadingFactor,
    /// LoRa Coding rate
    pub cr: CodingRate,
}

/// Spreading factor for LoRa mode
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[non_exhaustive]
pub enum SpreadingFactor {
    /// LoRa Spreading Factor 5, 32 chips / symbol
    Sf5,
    /// LoRa Spreading Factor 6, 64 chips / symbol
    Sf6,
    /// LoRa Spreading Factor 7, 128 chips / symbol
    Sf7,
    /// LoRa Spreading Factor 8, 256 chips / symbol
    Sf8,
    /// LoRa Spreading Factor 9, 512 chips / symbol
    Sf9,
    /// LoRa Spreading Factor 10 1024 chips / symbol
    Sf10,
    /// LoRa Spreading Factor 11 2048 chips / symbol
    Sf11,
    /// LoRa Spreading Factor 12 4096 chips / symbol
    Sf12,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[non_exhaustive]
pub enum CodingRate {
    /// LoRa Coding rate 4/5
    Cr4_5,
    /// LoRa Coding rate 4/6
    Cr4_6,
    /// LoRa Coding rate 4/7
    Cr4_7,
    /// LoRa Coding rate 4/8
    Cr4_8,
}
