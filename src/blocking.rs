//! Blocking APIs on top of the base radio traits
//! 
//! These implementations use the radio's DelayMs implementation to 
//! poll on completion of operations.
//! 
// https://github.com/ryankurte/rust-radio
// Copyright 2020 Ryan Kurte

use core::time::Duration;

use embedded_hal::blocking::delay::DelayMs;

use crate::{Transmit, Receive, Power, State};

pub struct BlockingOptions {
    pub power: Option<i8>,
    pub poll_interval: Duration,
    pub timeout: Duration,
}

impl Default for BlockingOptions {
    fn default() -> Self {
        Self {
            power: None,
            poll_interval: Duration::from_millis(1),
            timeout: Duration::from_millis(100),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BlockingError<E> {
    Inner(E),
    Timeout,
}

impl <E> From<E> for BlockingError<E> {
    fn from(e: E) -> Self {
        BlockingError::Inner(e)
    }
}

/// Blocking transmit function implemented over `radio::Transmit` and `radio::Power` using the provided 
/// `BlockingOptions` and radio-internal `DelayMs` impl to poll for completion
#[cfg_attr(feature = "mock", doc = r##"
```
# use radio::*;
# use radio::mock::*;
use radio::blocking::{BlockingTransmit, BlockingOptions};

# let mut radio = MockRadio::new(&[
#    Transaction::start_transmit(vec![0xaa, 0xbb], None),
#    Transaction::check_transmit(Ok(false)),
#    Transaction::delay_ms(1),
#    Transaction::check_transmit(Ok(true)),
# ]);
# 
// Transmit using a blocking call
let res = radio.do_transmit(&[0xaa, 0xbb], BlockingOptions::default());

assert_eq!(res, Ok(()));

# radio.done();
```
"##)]
///
pub trait BlockingTransmit<E> {
    fn do_transmit(&mut self, data: &[u8], tx_options: BlockingOptions) -> Result<(), BlockingError<E>>;
}

impl <T, E> BlockingTransmit<E> for T
where 
    T: Transmit<Error = E> + Power<Error = E> + DelayMs<u32>,
    E: core::fmt::Debug,
{
    fn do_transmit(&mut self, data: &[u8], tx_options: BlockingOptions) -> Result<(), BlockingError<E>> {
        // Set output power if specified
        if let Some(p) = tx_options.power {
            self.set_power(p)?;
        }

        self.start_transmit(data)?;

        let t = tx_options.timeout.as_millis();
        let mut c = 0;
        loop {
            if self.check_transmit()? {
                debug!("Blocking send complete");
                break;
            }
            
            c += tx_options.poll_interval.as_millis();
            if c > t {
                debug!("Blocking send timeout");
                return Err(BlockingError::Timeout)
            }

            self.delay_ms(tx_options.poll_interval.as_millis() as u32);
        }

        Ok(())
    }
}

/// Blocking receive function implemented over `radio::Receive` using the provided `BlockingOptions` 
/// and radio-internal `DelayMs` impl to poll for completion
#[cfg_attr(feature = "mock", doc = r##"
```
# use radio::*;
# use radio::mock::*;
use radio::blocking::{BlockingReceive, BlockingOptions};

let data = [0xaa, 0xbb];
let info = BasicInfo::new(-81, 0);


# let mut radio = MockRadio::new(&[
#    Transaction::start_receive(None),
#    Transaction::check_receive(true, Ok(false)),
#    Transaction::delay_ms(1),
#    Transaction::check_receive(true, Ok(true)),
#    Transaction::get_received(Ok((data.to_vec(), info.clone()))),
# ]);
# 

let mut buff = [0u8; 128];
let mut i = BasicInfo::new(0, 0);

// Receive using a blocking call
let res = radio.do_receive(&mut buff, &mut i, BlockingOptions::default());

assert_eq!(res, Ok(data.len()));
assert_eq!(&buff[..data.len()], &data);

# radio.done();
```
"##)]
/// 
pub trait BlockingReceive<I, E> {
    fn do_receive(&mut self, buff: &mut [u8], info: &mut I, rx_options: BlockingOptions) -> Result<usize, BlockingError<E>>;
}

impl <T, I, E> BlockingReceive<I, E> for T 
where
    T: Receive<Info=I, Error=E> + DelayMs<u32>,
    I: core::fmt::Debug,
    E: core::fmt::Debug,
{
    fn do_receive(&mut self, buff: &mut [u8], info: &mut I, rx_options: BlockingOptions) -> Result<usize, BlockingError<E>> {
        // Start receive mode
        self.start_receive()?;

        let t = rx_options.timeout.as_millis();
        let mut c = 0;
        loop {
            if self.check_receive(true)? {
                let n = self.get_received(info, buff)?;
                return Ok(n)
            }

            c += rx_options.poll_interval.as_millis();
            if c > t {
                debug!("Blocking receive timeout");
                return Err(BlockingError::Timeout)
            }

            self.delay_ms(rx_options.poll_interval.as_millis() as u32);
        }
    }
}

/// BlockingSetState sets the radio state and polls until command completion
pub trait BlockingSetState<S, E> {
    fn set_state_checked(&mut self, state: S, options: BlockingOptions) -> Result<(), BlockingError<E>>;
}

impl <T, S, E>BlockingSetState<S, E> for T 
where 
    T: State<State=S, Error=E> + DelayMs<u32>,
    S: core::fmt::Debug + core::cmp::PartialEq + Copy,
    E: core::fmt::Debug,
{
    fn set_state_checked(&mut self, state: S, options: BlockingOptions) -> Result<(), BlockingError<E>> {
        // Send set state command
        self.set_state(state)?;

        let t = options.timeout.as_millis();
        let mut c = 0;

        loop {
            // Fetch state
            let s = self.get_state()?;

            // Check for expected state
            if state == s {
                return Ok(())
            }

            // Timeout eventually
            c += options.poll_interval.as_millis();
            if c > t {
                debug!("Blocking receive timeout");
                return Err(BlockingError::Timeout)
            }

            // Delay before next loop
            self.delay_ms(options.poll_interval.as_millis() as u32);
        }

    }
}

