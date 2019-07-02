
use core::time::Duration;

use embedded_hal::blocking::delay::DelayMs;

use crate::{Transmit, Receive, Power};

/// Blocking transmit function implemented over `radio::Transmit` and `radio::Power` using the provided `DelayMs` impl to poll for completion
pub trait BlockingTransmit<E> {
    fn do_transmit<D>(&mut self, data: &[u8], power: Option<i8>, delay: &mut D, poll_interval: Duration) -> Result<(), E>
        where D: DelayMs<u32>;
}

impl <T, E> BlockingTransmit<E> for T
where 
    T: Transmit<Error = E> + Power<Error = E>,
    E: core::fmt::Debug,
{
    fn do_transmit<D>(&mut self, data: &[u8], power: Option<i8>, delay: &mut D, poll_interval: Duration) -> Result<(), E> 
    where D: DelayMs<u32>
    {
        // Set output power if specified
        if let Some(p) = power {
            self.set_power(p)?;
        }

        self.start_transmit(data)?;
        loop {
            if self.check_transmit()? {
                debug!("Send complete");
                break;
            }
            delay.delay_ms(poll_interval.as_millis() as u32);
        }

        Ok(())
    }
}

/// Blocking receive function implemented over `radio::Receive` using the provided `DelayMs` impl to poll for completion
pub trait BlockingReceive<I, E> {
    fn do_receive<D>(&mut self, buff: &mut [u8], info: &mut I, delay: &mut D, poll_interval: Duration) -> Result<usize, E>
        where D: DelayMs<u32>;
}

impl <T, I, E> BlockingReceive<I, E> for T 
where
    T: Receive<Info=I, Error=E>,
    I: core::fmt::Debug,
    E: core::fmt::Debug,
{
    fn do_receive<D>(&mut self, buff: &mut [u8], info: &mut I, delay: &mut D, poll_interval: Duration) -> Result<usize, E> 
    where
        D: DelayMs<u32>,
    {
        // Start receive mode
        self.start_receive()?;

        loop {
            if self.check_receive(true)? {
                let n = self.get_received(info, buff)?;
                return Ok(n)
            }
            delay.delay_ms(poll_interval.as_millis() as u32);
        }
    }
}
