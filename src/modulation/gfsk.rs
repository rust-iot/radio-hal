//! Common GFSK modulation options

/// Basic GFSK channel configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct GfskChannel {
    /// Channel frequency in kHz
    pub freq_khz: u32,

    /// Channel bandwidth in kHz
    pub bw_khz: u16,

    /// Bitrate in bps
    pub bitrate_bps: u32,
}
