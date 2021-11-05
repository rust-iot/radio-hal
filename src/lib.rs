//! Abstract packet radio interfaces
//!
//! This package defines traits for packet radio devices, as well as blocking and async
//! implementations using these traits, and a mock device to support application level testing.
//!
//! ## https://github.com/ryankurte/rust-radio
//! ## Copyright 2020 Ryan Kurte

#![no_std]

use core::fmt::Debug;

extern crate chrono;
extern crate nb;

#[macro_use]
extern crate log;

extern crate embedded_hal;

#[cfg(feature = "std")]
extern crate std;

pub mod config;

pub mod blocking;

#[cfg(feature = "helpers")]
pub mod helpers;
#[cfg(feature = "mock")]
pub mod mock;
#[cfg(feature = "nonblocking")]
pub mod nonblocking;

/// Radio trait combines Base, Configure, Send and Receive for a generic radio object
pub trait Radio: Transmit + Receive + State {}

/// Transmit trait for radios that can transmit packets
///
/// `start_transmit` should be called to load data into the radio, with `check_transmit` called
/// periodically (or using interrupts) to continue and finalise the transmission.
pub trait Transmit {
    /// Radio error
    type Error: Debug;

    /// Start sending a packet on the provided channel
    ///
    /// Returns an error if send was not started
    fn start_transmit(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Check for send completion
    ///
    /// Returns true for send complete, false otherwise
    fn check_transmit(&mut self) -> Result<bool, Self::Error>;
}

/// Receive trait for radios that can receive packets
///
/// `start_receive` should be used to setup the radio in receive mode, with `check_receive` called
/// periodically (or using interrupts) to poll for packet reception. Once a packet has been received,
/// `get_received` fetches the received packet (and associated info) from the radio.
pub trait Receive {
    /// Radio error
    type Error: Debug;
    /// Packet received info
    type Info: ReceiveInfo;

    /// Set receiving on the specified channel
    ///
    /// Returns an error if receive mode was not entered
    fn start_receive(&mut self) -> Result<(), Self::Error>;

    /// Check for reception
    ///
    /// The restart flag indicates on (recoverable) error conditions (such as invalid CRC)
    /// the radio should re-enter receive mode if required and continue reception.
    ///
    /// This returns true for received, false for not received, or the provided error
    fn check_receive(&mut self, restart: bool) -> Result<bool, Self::Error>;

    /// Fetch a received packet if rx is complete
    ///
    /// This copies received data into the provided buffer and returns the number of bytes received
    /// as well as information about the received packet
    fn get_received(&mut self, buff: &mut [u8]) -> Result<(usize, Self::Info), Self::Error>;
}

/// ReceiveInfo trait for receive information objects
///
/// This sup[ports the constraint of generic `Receive::Info`, allowing generic middleware
/// to access the rssi of received packets
pub trait ReceiveInfo: Debug + Default {
    fn rssi(&self) -> i16;
}

/// Default / Standard packet information structure for radio devices that provide only rssi
/// and lqi information
#[derive(Debug, Clone, PartialEq)]
pub struct BasicInfo {
    /// Received Signal Strength Indicator (RSSI) of received packet in dBm
    rssi: i16,
    /// Link Quality Indicator (LQI) of received packet
    lqi: u16,
}

impl Default for BasicInfo {
    fn default() -> Self {
        Self {
            rssi: core::i16::MIN,
            lqi: core::u16::MIN,
        }
    }
}

impl BasicInfo {
    pub fn new(rssi: i16, lqi: u16) -> Self {
        Self { rssi, lqi }
    }
}

/// Default / Standard radio channel object for radio devices with simple integer channels
impl ReceiveInfo for BasicInfo {
    fn rssi(&self) -> i16 {
        self.rssi
    }
}

/// Default / Standard radio channel object for radio devices with integer channels
#[derive(Debug, Clone, PartialEq)]
pub struct BasicChannel(pub u16);

impl From<u16> for BasicChannel {
    fn from(u: u16) -> Self {
        BasicChannel(u)
    }
}

impl Into<u16> for BasicChannel {
    fn into(self) -> u16 {
        self.0
    }
}

/// Channel trait for configuring radio channelization
pub trait Channel {
    /// Channel information
    type Channel: Debug;
    /// Radio error type
    type Error: Debug;

    /// Set the radio channel for future transmit and receive operations
    fn set_channel(&mut self, channel: &Self::Channel) -> Result<(), Self::Error>;
}

/// Power trait for configuring radio power
pub trait Power {
    /// Radio error type
    type Error: Debug;

    /// Set the radio power in dBm
    fn set_power(&mut self, power: i8) -> Result<(), Self::Error>;
}

/// Rssi trait allows polling for RSSI on the current channel
///
/// Note that the radio should be in receive mode prior to polling for this.
pub trait Rssi {
    /// Radio error
    type Error: Debug;

    /// Fetch the current RSSI value from the radio
    /// Note that the radio MUST be in RX mode (or capable of measuring RSSI) when this is called
    /// or an error should be returned
    fn poll_rssi(&mut self) -> Result<i16, Self::Error>;
}

/// State trait for configuring and reading radio states
///
/// Note that drivers will internally configure and read radio states to manage
/// radio operations.
pub trait State {
    /// Radio state
    type State: RadioState;
    /// Radio error type
    type Error: Debug;

    /// Set the radio to a specified state
    fn set_state(&mut self, state: Self::State) -> Result<(), Self::Error>;

    /// Fetch the current radio state
    fn get_state(&mut self) -> Result<Self::State, Self::Error>;
}

pub trait RadioState: Debug {
    fn idle() -> Self;

    fn sleep() -> Self;
}

/// Busy trait for checking whether the radio is currently busy
/// and should not be interrupted
pub trait Busy {
    /// Radio error type
    type Error: Debug;

    /// Indicates the radio is busy in the current state
    /// (for example, currently transmitting or receiving)
    fn is_busy(&mut self) -> Result<bool, Self::Error>;
}

/// Interrupts trait allows for reading interrupt state from the device,
/// as well as configuring interrupt pins.
///
/// Note that drivers may internally use interrupts and interrupt states
/// to manage radio operations.
pub trait Interrupts {
    /// Interrupt object
    type Irq: Debug;
    /// Radio error
    type Error: Debug;

    /// Fetch any pending interrupts from the device
    /// If the clear option is set, this will also clear any returned flags
    fn get_interrupts(&mut self, clear: bool) -> Result<Self::Irq, Self::Error>;
}

/// Register contains the address and value of a register.
///
/// It is primarily intended as a type constraint for the [Registers] trait.
pub trait Register: Copy + From<u8> + Into<u8> {
    const ADDRESS: u8;
}

/// Registers trait provides register level access to the radio device.
///
/// This is generally too low level for use by higher abstractions, however,
/// is provided for completeness.
pub trait Registers {
    type Error: Debug;

    /// Read a register value
    fn read_register<R: Register>(&mut self) -> Result<R, Self::Error>;

    /// Write a register value
    fn write_register<R: Register>(&mut self, value: R) -> Result<(), Self::Error>;

    /// Update a register value
    fn update_register<R: Register>(&mut self, f: fn(R) -> R) -> Result<R, Self::Error> {
        let existing = self.read_register()?;
        let updated = f(existing);
        self.write_register(updated)?;
        Ok(updated)
    }
}

#[cfg(feature = "structopt")]
use crate::std::str::FromStr;

#[cfg(feature = "structopt")]
fn duration_from_str(s: &str) -> Result<core::time::Duration, humantime::DurationError> {
    let d = humantime::Duration::from_str(s)?;
    Ok(*d)
}

#[cfg(test)]
mod tests {
    use crate::{Register, Registers};

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct TestRegister {
        value: u8,
    }

    impl From<u8> for TestRegister {
        fn from(value: u8) -> Self {
            Self { value }
        }
    }

    impl From<TestRegister> for u8 {
        fn from(reg: TestRegister) -> Self {
            reg.value
        }
    }

    impl Register for TestRegister {
        const ADDRESS: u8 = 123;
    }

    struct TestDevice {
        device_register: u8,
    }

    impl Registers for TestDevice {
        type Error = ();
        fn read_register<R: Register>(&mut self) -> Result<R, Self::Error> {
            if R::ADDRESS == TestRegister::ADDRESS {
                Ok(self.device_register.into())
            } else {
                Err(())
            }
        }

        fn write_register<R: Register>(&mut self, value: R) -> Result<(), Self::Error> {
            if R::ADDRESS == TestRegister::ADDRESS {
                self.device_register = value.into();
                Ok(())
            } else {
                Err(())
            }
        }
    }

    #[test]
    fn update_register() {
        let mut device = TestDevice { device_register: 0 };
        device.write_register(TestRegister { value: 1 }).unwrap();
        device
            .update_register(|r: TestRegister| (if r.value == 1 { 2 } else { 3 }).into())
            .unwrap();
        assert_eq!(
            device.read_register::<TestRegister>().unwrap(),
            TestRegister { value: 2 }
        );
    }
}
