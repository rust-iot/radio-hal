//! Abstract packet radio interfaces
//! 
//! https://github.com/rust-iot/radio
//! 
// Copyright 2018 Ryan Kurte

#![no_std]
#![deny(unsafe_code)]

extern crate nb;

#[macro_use]
extern crate log;

extern crate embedded_hal;

pub mod blocking;

/// Radio trait combines Base, Configure, Send and Receive for a generic radio object
pub trait Radio: Transmit + Receive {}

/// Send trait for radios that can transmit packets
pub trait Transmit {
    /// Radio error
    type Error;

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
pub trait Receive {
    /// Radio error
    type Error;
    /// Packet received info
    type Info;

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
    fn get_received<'a>(&mut self, &mut Self::Info, &'a mut [u8]) -> Result<usize, Self::Error>;
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

/// State trait for configuring radio states
pub trait State {
    /// Channel information
    type State;
    /// Radio error type
    type Error;

    /// Set the radio to a specified state
    fn set_state(&mut self, state: Self::State) -> Result<(), Self::Error>;

    /// Fetch the current radio state
    fn get_state(&mut self) -> Result<Self::State, Self::Error>;
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
pub trait Rssi {
    /// Radio error
    type Error;
    
    /// Fetch the current RSSI value from the radio
    /// Note that the radio MUST be in RX mode (or capable of measuring RSSI) when this is called
    /// or an error should be returned
    fn poll_rssi(&mut self) -> Result<i16, Self::Error>;
}

/// Rssi trait allows polling for RSSI on the current channel
pub trait Interrupts {
    /// Interrupt object
    type Irq;
    /// Radio error
    type Error;
    
    /// Fetch any pending interrupts from the device
    /// If the clear option is set, this will also clear any returned flags
    fn get_interrupts(&mut self, clear: bool) -> Result<Self::Irq, Self::Error>;
}

/// Registers trait provides register level access to the radio device
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
