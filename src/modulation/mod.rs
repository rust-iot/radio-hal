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
    UnsupportedFreq,
    UnsupportedBandwidth,
}

/// Basic frequency type for use in radio definitions.
///
/// This splits frequencies into integer `khz` and `hz` components to achieve Hz granularity with >>GHz range.
/// Users above ~4 GHz should prefer [`Freq::parts`] over integer conversions.
/// ```rust
/// # use radio::modulation::{Freq, Frequency};
/// // Freq objects can be constructed from numeric types
/// let freq = 434.mhz();
/// assert_eq!(freq, 434_000.khz());
/// // And converted back into these numeric types as required
/// assert_eq!(434, freq.mhz());
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Freq {
    /// kHz component
    khz: u32,
    /// Hz portion (0-1000)
    hz: u32,
}

/// Frequency trait for type conversions.
/// See [`Freq`] implementations for details
pub trait Frequency<T> {
    fn hz(&self) -> T;
    fn khz(&self) -> T;
    fn mhz(&self) -> T;
    fn ghz(&self) -> T;
}

impl Freq {
    /// Create a new frequency from kHz and Hz components
    pub const fn from_parts(khz: u32, hz: u32) -> Option<Self> {
        if hz >= 1000 {
            return None;
        }
        Some(Self { khz, hz })
    }

    /// Fetch frequency kHz and Hz components
    pub const fn parts(&self) -> (u32, u32) {
        (self.khz, self.hz)
    }
}

/// Fetch u32 values from [`Freq`] types
impl Frequency<u32> for Freq {
    /// Convert [`Freq`] to u32 Hz, note this will panic for frequencies over ~4GHz
    /// ```
    /// # use radio::modulation::{Freq, Frequency};
    /// let f = Freq::from_parts(433_100, 200).unwrap();
    /// assert_eq!(f.hz(), 433_100_200);
    /// ```
    fn hz(&self) -> u32 {
        self.khz * 1000 + self.hz
    }

    /// Convert [`Freq`] to integer kHz
    /// ```
    /// # use radio::modulation::{Freq, Frequency};
    /// let f = Freq::from_parts(433_100, 200).unwrap();
    /// assert_eq!(f.khz(), 433_100);
    /// ```
    fn khz(&self) -> u32 {
        self.khz
    }

    /// Convert [`Freq`] to integer MHz
    /// ```
    /// # use radio::modulation::{Freq, Frequency};
    /// let f = Freq::from_parts(2_400_100, 200).unwrap();
    /// assert_eq!(f.mhz(), 2_400);
    /// ```
    fn mhz(&self) -> u32 {
        self.khz() / 1000
    }

    /// Convert [`Freq`] to integer GHz
    /// ```
    /// # use radio::modulation::{Freq, Frequency};
    /// let f = Freq::from_parts(20_000_000, 200).unwrap();
    /// assert_eq!(f.ghz(), 20);
    /// ```
    fn ghz(&self) -> u32 {
        self.mhz() / 1000
    }
}

/// Create [`Freq`] objects from [`u32`] frequencies
impl Frequency<Freq> for u32 {
    /// Create [`Freq`] from integer Hz
    /// ```
    /// # use radio::modulation::{Freq, Frequency};
    /// let f = 434_100_200.hz();
    /// assert_eq!(f.khz(), 434_100);
    /// assert_eq!(f.hz(), 434_100_200);
    /// ```
    fn hz(&self) -> Freq {
        Freq {
            khz: self / 1000,
            hz: self % 1000,
        }
    }

    /// Create [`Freq`] from integer kHz
    /// ```
    /// # use radio::modulation::{Freq, Frequency};
    /// let f = 434_100.khz();
    /// assert_eq!(f.hz(), 434_100_000);
    /// ```
    fn khz(&self) -> Freq {
        Freq { khz: *self, hz: 0 }
    }

    /// Create [`Freq`] from integer MHz
    /// ```
    /// # use radio::modulation::{Freq, Frequency};
    /// let f = 2_450.mhz();
    /// assert_eq!(f.khz(), 2_450_000);
    /// ```
    fn mhz(&self) -> Freq {
        Freq {
            khz: self * 1000,
            hz: 0,
        }
    }

    /// Create [`Freq`] from integer GHz
    /// ```
    /// # use radio::modulation::{Freq, Frequency};
    /// let f = 2.ghz();
    /// assert_eq!(f.mhz(), 2_000);
    /// ```
    fn ghz(&self) -> Freq {
        Freq {
            khz: self * 1000 * 1000,
            hz: 0,
        }
    }
}
