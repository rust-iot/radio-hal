//! Config provides traits for standard radio configuration

/// Radio configuration options
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
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

/// Radio configuration errors
/// This should be extended with errors generally relevant to configuration,
/// with radio-specific errors passed through the Other(E) field.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum ConfigError<E> {
    /// Configuration option not supported
    NotSupported,

    /// Other (device, non-configuration errors)
    Other(E),
}

/// Configure trait implemented by configurable radios
pub trait Configure {
    /// Radio error
    type Error;

    /// Set a configuration option
    /// Returns Ok(true) on set, Ok(false) for unsupported options, Err(Self::Error) for errors
    fn set_option(&mut self, o: &ConfigOption) -> Result<(), ConfigError<Self::Error>>;

    /// Fetch a configuration option
    /// This will overwrite the value of the provided option enum
    /// Returns Ok(true) on successful get, Ok(false) for unsupported options, Err(Self::Error) for errors
    fn get_option(&mut self, o: &mut ConfigOption) -> Result<(), ConfigError<Self::Error>>;
}
