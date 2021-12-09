//! Common GFSK modulation options

use super::Freq;

/// Basic GFSK channel configuration
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct GfskChannel {
    /// Channel frequency
    pub freq: Freq,

    /// Channel bandwidth
    pub bw_khz: Freq,

    /// Bitrate in bps
    pub bitrate_bps: u32,
}
