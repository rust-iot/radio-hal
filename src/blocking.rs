
use core::time::Duration;

use embedded_hal::blocking::delay::DelayMs;

use crate::{Transmit, Receive, Power};

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

/// Blocking transmit function implemented over `radio::Transmit` and `radio::Power` using the provided `DelayMs` impl to poll for completion
pub trait BlockingTransmit<E> {
    fn do_transmit<D>(&mut self, data: &[u8], tx_options: BlockingOptions) -> Result<(), BlockingError<E>>;
}

impl <T, E> BlockingTransmit<E> for T
where 
    T: Transmit<Error = E> + Power<Error = E> + DelayMs<u32>,
    E: core::fmt::Debug,
{
    fn do_transmit<D>(&mut self, data: &[u8], tx_options: BlockingOptions) -> Result<(), BlockingError<E>> {
        // Set output power if specified
        if let Some(p) = tx_options.power {
            self.set_power(p)?;
        }

        self.start_transmit(data)?;

        let t = tx_options.timeout.as_millis();
        let mut c = 0;
        loop {
            if self.check_transmit()? {
                debug!("Send complete");
                break;
            }
            
            c += tx_options.poll_interval.as_millis();
            if c > t {
                debug!("Send timeout");
                return Err(BlockingError::Timeout)
            }

            self.delay_ms(tx_options.poll_interval.as_millis() as u32);
        }

        Ok(())
    }
}

/// Blocking receive function implemented over `radio::Receive` using the provided `DelayMs` impl to poll for completion
pub trait BlockingReceive<I, E> {
    fn do_receive<D>(&mut self, buff: &mut [u8], info: &mut I, rx_options: BlockingOptions) -> Result<usize, BlockingError<E>>;
}

impl <T, I, E> BlockingReceive<I, E> for T 
where
    T: Receive<Info=I, Error=E> + DelayMs<u32>,
    I: core::fmt::Debug,
    E: core::fmt::Debug,
{
    fn do_receive<D>(&mut self, buff: &mut [u8], info: &mut I, rx_options: BlockingOptions) -> Result<usize, BlockingError<E>> {
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
                debug!("Send timeout");
                return Err(BlockingError::Timeout)
            }

            self.delay_ms(rx_options.poll_interval.as_millis() as u32);
        }
    }
}
