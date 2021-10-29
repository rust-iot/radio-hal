//! Abstract packet radio interfaces
//! 
//! This package defines traits for packet radio devices, as well as blocking and async
//! implementations using these traits, and a mock device to support application level testing.
//! 
//! ## https://github.com/ryankurte/rust-radio
//! ## Copyright 2020 Ryan Kurte

#![no_std]

extern crate nb;
extern crate chrono;

#[macro_use]
extern crate log;

extern crate embedded_hal;

#[cfg(feature="std")]
extern crate std;

pub mod config;

pub mod blocking;

pub mod params;

#[cfg(feature="nonblocking")]
pub mod nonblocking;
#[cfg(feature="helpers")]
pub mod helpers;
#[cfg(feature="mock")]
pub mod mock;

/// Radio trait combines Base, Configure, Send and Receive for a generic radio object
pub trait Radio<P: Param>: Transmit<P> + Receive<P> + State {}

/// Transmit trait for radios that can transmit packets
/// 
/// `start_transmit` should be called to load data into the radio, with `check_transmit` called
/// periodically (or using interrupts) to continue and finalise the transmission.
pub trait Transmit<P> {
    /// Radio error
    type Error;

    /// Start sending a packet on the provided channel
    /// 
    /// Returns an error if send was not started
    fn start_transmit(&mut self, data: &[u8], params: &P) -> Result<(), Self::Error>;

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
pub trait Receive<P: Param> {
    /// Radio error
    type Error;

    /// Set receiving on the specified channel
    /// 
    /// Returns an error if receive mode was not entered
    fn start_receive(&mut self, params: &P) -> Result<(), Self::Error>;

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
    fn get_received(&mut self, info: &mut P::Info, buff: &mut [u8]) -> Result<usize, Self::Error>;
}

/// ReceiveInfo trait for receive information objects
/// 
/// This sup[ports the constraint of generic `Receive::Info`, allowing generic middleware
/// to access the rssi of received packets
pub trait ReceiveInfo {
    fn rssi(&self) -> i16;
}

/// Default / Standard packet information structure for radio devices that provide only rssi 
/// and lqi information
#[derive(Debug, Clone, PartialEq)]
pub struct BasicInfo {
    /// Received Signal Strength Indicator (RSSI) of received packet in dBm
    rssi:   i16,
    /// Link Quality Indicator (LQI) of received packet  
    lqi:    u16,
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
        Self {rssi, lqi}
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
pub struct BasicChannel (pub u16);

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
    type Channel;
    /// Radio error type
    type Error;

    /// Set the radio channel for future transmit and receive operations
    fn set_channel(&mut self, channel: &Self::Channel) -> Result<(), Self::Error>;
}

/// Power trait for configuring radio power
pub trait Power {
    /// Radio error type
    type Error;

    /// Set the radio power in dBm
    fn set_power(&mut self, power: i8) -> Result<(), Self::Error>;
}

/// Rssi trait allows polling for RSSI on the current channel
/// 
/// Note that the radio should be in receive mode prior to polling for this.
pub trait Rssi {
    /// Radio error
    type Error;
    
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
    type State;
    /// Radio error type
    type Error;

    /// Set the radio to a specified state
    fn set_state(&mut self, state: Self::State) -> Result<(), Self::Error>;

    /// Fetch the current radio state
    fn get_state(&mut self) -> Result<Self::State, Self::Error>;
}

pub trait RadioState {
    fn idle() -> Self;

    fn sleep() -> Self;
}

/// Busy trait for checking whether the radio is currently busy
/// and should not be interrupted
pub trait Busy {
    /// Radio error type
    type Error;

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
    type Irq;
    /// Radio error
    type Error;
    
    /// Fetch any pending interrupts from the device
    /// If the clear option is set, this will also clear any returned flags
    fn get_interrupts(&mut self, clear: bool) -> Result<Self::Irq, Self::Error>;
}

/// Registers trait provides register level access to the radio device.
/// 
/// This is generally too low level for use by higher abstractions, however,
/// is provided for completeness.
pub trait Registers<R: Copy> {
    type Error;

    /// Read a register value
    fn reg_read(&mut self, reg: R) -> Result<u8, Self::Error>;

    /// Write a register value
    fn reg_write(&mut self, reg: R, value: u8) -> Result<(), Self::Error>;
    
    /// Update a register value
    fn reg_update(&mut self, reg: R, mask: u8, value: u8) -> Result<u8, Self::Error> {
        let existing = self.reg_read(reg)?;
        let updated = (existing & !mask) | (value & mask);
        self.reg_write(reg, updated)?;
        Ok(updated)
    }
}

use crate::params::Param;
#[cfg(feature="structopt")]
use crate::std::str::FromStr;

#[cfg(feature="structopt")]
fn duration_from_str(s: &str) -> Result<core::time::Duration, humantime::DurationError> {
    let d = humantime::Duration::from_str(s)?;
    Ok(*d)
}
