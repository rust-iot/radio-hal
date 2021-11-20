//! Shared types for radio channel / modulation configuration

use core::fmt::Debug;

pub mod gfsk;

pub mod lora;

/// Common modulation configuration errors
///
/// These are provided as a helper for `TryFrom` implementations,
/// and not intended to be prescriptive.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ModError {
    UnsupportedBitrate,
    UnsupportedFrequency,
    UnsupportedBandwidth,
}
