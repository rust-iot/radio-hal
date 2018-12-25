//! Abstract packet radio interfaces
//! rust-iot
//! 
//! Copyright 2018 Ryan Kurte

#![no_std]
#![deny(unsafe_code)]

/// Radio trait combines Base, Configure, Send and Receive for a generic radio object
pub trait Radio: Transmit + Receive {}

/// Send trait for radios that can transmit packets
pub trait Transmit {
    /// Radio error
    type Error;

    /// Start sending a packet on the provided channel
    /// Returns an error if send was not started
    fn start_transmit(&mut self, channel: u16, data: &[u8]) -> Result<(), Self::Error>;

    /// Check for send completion
    /// Returns true for send complete, false otherwise
    fn check_transmit(&mut self) -> Result<bool, Self::Error>;
}

/// Default / Standard packet information structure
/// This may be used by radio::Receive implementors
#[derive(Debug)]
pub struct Info {
    rssi:   i16,  // Received Signal Strength Indicator (RSSI) of received packet in dBm
    lqi:    u16   // Link Quality Indicator (LQI) of received packet
}

/// Receive trait for radios that can receive packets
pub trait Receive {
    /// Radio error
    type Error;
    /// Packet received info
    type Info;

    /// Set receiving on the specified channel
    /// Returns an error if receive mode was not entered
    fn start_receive(&mut self, channel: u16) -> Result<(), Self::Error>;

    /// Fetch a received packet if rx is complete
    /// Returns Some on complete, None while still receiving, or an error on failure
    fn get_received<'a>(&mut self, &'a mut [u8]) -> Result<Option<(&'a[u8], Self::Info)>, Self::Error>;

}

/// RSSI trait allows polling for RSSI on the current channel (when in receive mode)
pub trait Rssi {
    /// Radio error
    type Error;
    
    /// Fetch the current RSSI value from the radio
    fn get_rssi(&mut self) -> Result<i16, Self::Error>;
}

/// Registers trait provides register level access to the radio device
pub trait Registers {
    type Register;
    type Error;

    /// Read a register value
    fn reg_read<'a>(&mut self, reg: Self::Register) -> Result<u8, Self::Error>;

    /// Write a register value
    fn reg_write(&mut self, reg: Self::Register, value: u8) -> Result<(), Self::Error>;
    
    /// Update a register value
    fn reg_update(&mut self, reg: Self::Register, mask: u8, value: u8) -> Result<(), Self::Error>;

}