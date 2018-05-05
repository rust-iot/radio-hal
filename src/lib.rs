//! Abstract radio interface

#![no_std]
#![deny(unsafe_code)]

extern crate managed;
#[cfg(any(test, feature = "std"))]
extern crate std;
extern crate nb;

use self::std::boxed::Box;

/// Basic radio states
pub enum State {
    /// Radio powered off
    OFF             = 0,
    /// Radio IDLE (awaiting commands)    
    IDLE            = 1,
    /// Radio sleeping (low power mode)    
    SLEEP           = 2,
    /// In receive mode (not receiving)
    RECEIVE         = 3,
    /// In receive mode (receiving)
    RECEIVING       = 4,
    /// Currently transmitting
    TRANSMITTING    = 5
}

/// Radio trait combines Base, Configure, Send and Receive for a generic radio object
pub trait Radio: Base + Configure + Send + Receive {}

/// Base radio traits
pub trait Base {
    /// Radio error
    type Error;

    /// Fetch the current radio state
    fn get_state(&mut Self) -> nb::Result<State, Self::Error>;
    /// Set the ratio state
    fn set_state(&mut Self, State) -> nb::Result<State, Self::Error>;
}

/// Basic radio options
pub enum ConfigOption {
    /// MAC address
    MAC([u8; 6]),
    /// IPv4 address
    IPv4([u8; 4]),
    /// IPv6 address
    IPv6([u8; 16]),

    /// IEEE802.15.4(g) / ZigBee address options
    /// Short (16-bit) Address
    ShortAddress(u16),
    /// Long (64-bit) Address
    LongAddress(u64),
    /// PAN ID
    PAN(u16),

    /// Maximum Transmission Unit (MTU)
    MTU(u16),
    /// Transmit power (dBm)
    TXPower(i16),


    /// Await Clear Channel before TX (if supported)
    AwaitCCA(bool),
    /// CCA threshold in dBm (used if AwaitCCA is set)
    CCAThreshold(i16),
    /// Automatic Acknowledgement (if supported) sends 802.15.4 acknowledgements automatically
    AutoAck(bool),
    /// Promiscuous mode (if supported) disables hardware address filtering
    Promiscuous(bool),
}

/// Configure trait implemented by configurable radios
pub trait Configure {
    /// Radio error
    type Error;

    /// Set a configuration option
    fn set_option(&mut Self, o: &ConfigOption) -> nb::Result<(), Self::Error>;

    /// Fetch a configuration option
    /// This will overwrite the value of the provided option enum
    fn get_option(&mut Self, o: &mut ConfigOption) -> nb::Result<(), Self::Error>;
}

/// Send trait for radios that can transmit packets
pub trait Send {
    /// Radio error
    type Error;

    /// Start sending a packet on the provided channel
    fn start_send(&mut Self, channel: u16, data: &[u8]) -> nb::Result<State, Self::Error>;
    /// Check for send completion
    fn check_send(&mut Self) -> nb::Result<State, Self::Error>;
}

/// Default packet information structure
#[derive(Debug)]
pub struct Info {
    rssi:   i16,  // Received Signal Strength Indicator (RSSI) of received packet in dBm
    lqi:    u16   // Link Quality Indicator (LQI) of received packet
}

/// Receive trait for radios that can receive packets
pub trait Receive {
    /// Radio error
    type Error;

    /// Set receiving on the specified channel
    fn start_receive(&mut Self, channel: u16) -> nb::Result<State, Self::Error>;
    /// Fetch a received packet if rx is complete
    fn get_received<'a>(&mut Self) -> nb::Result<Option<(&'a[u8], Info)>, Self::Error>;

    /// Fetch the current RSSI value from the radio
    fn get_rssi(&mut Self) -> nb::Result<i16, Self::Error>;
}


/// Events for async callbacks
pub enum Event {
    /// Radio state changed event
    StateChange(State),
    /// Channel hop event
    ChannelHop(u16),
    /// Transmission completed
    TXComplete,
    /// No ack for transmission
    TXErrorNoAck,
    /// CCA timeout attempting transmission
    TXErrorCCATimeout,
    /// Generic transmission error
    TXError(isize),
    /// Started receiving a packet
    RXStart,
    /// Packet received
    RXComplete,
    /// Receive CRC error
    RXErrorCRC,
    /// Receive timeout
    RXErrorTimeout,
    /// Generic receive error
    RXError(isize),
}

/// Async trait implemented by radios with interrupt driven state changes
/// This callback should only be called on interrupt events
pub trait Async<T> {
    /// Set radio event callback
    /// Called when radio state changes, packets finish sending etc. to handle async / interrupt driven state changes
    fn set_callback<CB: 'static + FnMut(T, Event)>(&mut self, callback: CB);
}

/// AsyncHelper provides callback binding and calling helpers for radio implementations
pub struct AsyncHelper<T> {
    callback: Box<FnMut(T, Event)>
}

impl <T> Async<T> for AsyncHelper<T> {
    fn set_callback<CB: 'static + FnMut(T, Event)>(&mut self, c: CB) {
        self.callback = Box::new(c);
    }
}

impl <T> AsyncHelper<T> {
    /// Execute the bound callback in the sync helper
    pub fn do_callback(&mut self, ctx: T, e: Event) {
        (self.callback)(ctx, e);
    }
}
