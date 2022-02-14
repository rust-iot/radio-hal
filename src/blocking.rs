//! Blocking APIs on top of the base radio traits
//!
//! These implementations use the radio's DelayUs implementation to
//! poll on completion of operations.
//!
//! ## https://github.com/ryankurte/rust-radio
//! ## Copyright 2020 Ryan Kurte

use core::fmt::Debug;
use core::time::Duration;

use embedded_hal::delay::blocking::DelayUs;

#[cfg(not(feature = "defmt"))]
use log::debug;

#[cfg(feature = "defmt")]
use defmt::debug;

#[cfg(feature = "structopt")]
use structopt::StructOpt;

#[cfg(feature = "std")]
use std::string::ToString;

use crate::{Receive, State, Transmit};

/// BlockingOptions for blocking radio functions
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "structopt", derive(StructOpt))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BlockingOptions {
    /// Interval for polling for device state
    #[cfg_attr(feature="structopt", structopt(long, default_value="100us", parse(try_from_str=crate::duration_from_str)))]
    pub poll_interval: Duration,

    /// Timeout for blocking operation
    #[cfg_attr(feature="structopt", structopt(long, default_value="100ms", parse(try_from_str=crate::duration_from_str)))]
    pub timeout: Duration,
}

impl Default for BlockingOptions {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_micros(100),
            timeout: Duration::from_millis(100),
        }
    }
}

/// BlockingError wraps radio error type to provie a `Timeout` variant
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BlockingError<E> {
    #[cfg_attr(feature = "thiserror", error("Inner: {0}"))]
    Inner(E),
    #[cfg_attr(feature = "thiserror", error("Timeout"))]
    Timeout,
}

impl<E> From<E> for BlockingError<E> {
    fn from(e: E) -> Self {
        BlockingError::Inner(e)
    }
}

/// Blocking transmit function implemented over `radio::Transmit` and `radio::Power` using the provided
/// `BlockingOptions` and radio-internal `DelayUs` impl to poll for completion
#[cfg_attr(
    feature = "mock",
    doc = r##"
```
# use radio::*;
# use radio::mock::*;
use radio::blocking::{BlockingTransmit, BlockingOptions};

# let mut radio = MockRadio::new(&[
#    Transaction::start_transmit(vec![0xaa, 0xbb], None),
#    Transaction::check_transmit(Ok(false)),
#    Transaction::delay_us(100),
#    Transaction::check_transmit(Ok(true)),
# ]);
# 
// Transmit using a blocking call
let res = radio.do_transmit(&[0xaa, 0xbb], BlockingOptions::default());

assert_eq!(res, Ok(()));

# radio.done();
```
"##
)]
///
pub trait BlockingTransmit<E: Debug> {
    fn do_transmit(
        &mut self,
        data: &[u8],
        tx_options: BlockingOptions,
    ) -> Result<(), BlockingError<E>>;
}

impl<T, E> BlockingTransmit<E> for T
where
    T: Transmit<Error = E> + DelayUs,
    E: Debug,
{
    fn do_transmit(
        &mut self,
        data: &[u8],
        tx_options: BlockingOptions,
    ) -> Result<(), BlockingError<E>> {
        // Enter transmit mode
        self.start_transmit(data)?;

        let t = tx_options.timeout.as_micros();
        let mut c = 0;
        loop {
            // Check for transmit complete
            if self.check_transmit()? {
                #[cfg(any(feature = "log", feature = "defmt"))]
                debug!("Blocking send complete");
                break;
            }

            // Update poll time and timeout if overrun
            c += tx_options.poll_interval.as_micros();
            if c > t {
                #[cfg(any(feature = "log", feature = "defmt"))]
                debug!("Blocking send timeout");
                return Err(BlockingError::Timeout);
            }

            // Wait for next poll
            let _ = self.delay_us(tx_options.poll_interval.as_micros() as u32);
        }

        Ok(())
    }
}

/// Blocking receive function implemented over `radio::Receive` using the provided `BlockingOptions`
/// and radio-internal `DelayUs` impl to poll for completion
#[cfg_attr(
    feature = "mock",
    doc = r##"
```
# use radio::*;
# use radio::mock::*;
use radio::blocking::{BlockingReceive, BlockingOptions};

let data = [0xaa, 0xbb];
let info = BasicInfo::new(-81, 0);

# let mut radio = MockRadio::new(&[
#    Transaction::start_receive(None),
#    Transaction::check_receive(true, Ok(false)),
#    Transaction::delay_us(100),
#    Transaction::check_receive(true, Ok(true)),
#    Transaction::get_received(Ok((data.to_vec(), info.clone()))),
# ]);
# 

// Setup buffer to read into
let mut buff = [0u8; 128];

// Receive using a blocking call
let (n, info) = radio.do_receive(&mut buff, BlockingOptions::default())?;

assert_eq!(n, data.len());
assert_eq!(&buff[..data.len()], &data);

# radio.done();

# Ok::<(), anyhow::Error>(())
```
"##
)]
///
pub trait BlockingReceive<I, E> {
    fn do_receive(
        &mut self,
        buff: &mut [u8],
        rx_options: BlockingOptions,
    ) -> Result<(usize, I), BlockingError<E>>;
}

impl<T, I, E> BlockingReceive<I, E> for T
where
    T: Receive<Info = I, Error = E> + DelayUs,
    <T as Receive>::Info: Debug,
    I: Debug,
    E: Debug,
{
    fn do_receive(
        &mut self,
        buff: &mut [u8],
        rx_options: BlockingOptions,
    ) -> Result<(usize, I), BlockingError<E>> {
        // Start receive mode
        self.start_receive()?;

        let t = rx_options.timeout.as_micros();
        let mut c = 0;
        loop {
            if self.check_receive(true)? {
                let (n, i) = self.get_received(buff)?;
                return Ok((n, i));
            }

            c += rx_options.poll_interval.as_micros();
            if c > t {
                #[cfg(any(feature = "log", feature = "defmt"))]
                debug!("Blocking receive timeout");
                return Err(BlockingError::Timeout);
            }

            let _ = self.delay_us(rx_options.poll_interval.as_micros() as u32);
        }
    }
}

/// BlockingSetState sets the radio state and polls until command completion
pub trait BlockingSetState<S, E> {
    fn set_state_checked(
        &mut self,
        state: S,
        options: BlockingOptions,
    ) -> Result<(), BlockingError<E>>;
}

impl<T, S, E> BlockingSetState<S, E> for T
where
    T: State<State = S, Error = E> + DelayUs,
    S: Debug + core::cmp::PartialEq + Copy,
    E: Debug,
{
    fn set_state_checked(
        &mut self,
        state: S,
        options: BlockingOptions,
    ) -> Result<(), BlockingError<E>> {
        // Send set state command
        self.set_state(state)?;

        let t = options.timeout.as_micros();
        let mut c = 0;

        loop {
            // Fetch state
            let s = self.get_state()?;

            // Check for expected state
            if state == s {
                return Ok(());
            }

            // Timeout eventually
            c += options.poll_interval.as_micros();
            if c > t {
                #[cfg(any(feature = "log", feature = "defmt"))]
                debug!("Blocking receive timeout");
                return Err(BlockingError::Timeout);
            }

            // Delay before next loop
            let _ = self.delay_us(options.poll_interval.as_micros() as u32);
        }
    }
}
