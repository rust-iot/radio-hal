
use core::time::Duration;

extern crate embedded_hal;
use blocking::embedded_hal::blocking::delay::DelayMs;

use crate::{Transmit, Receive, Power};

/// Blocking transmit function that uses thread_sleep internally to transmit a packet
pub fn do_transmit<T, D, E>(radio: &mut T, data: &[u8], power: Option<i8>, mut delay: D, poll_interval: Duration) -> Result<(), E> 
where
    T: Transmit<Error=E> + Power<Error=E>,
    D: DelayMs<u32>,
{
    // Set output power if specified
    if let Some(p) = power {
        radio.set_power(p)?;
    }

    radio.start_transmit(data)?;
    loop {
        if radio.check_transmit()? {
            debug!("Send complete");
            break;
        }
        delay.delay_ms(poll_interval.as_millis() as u32);
    }

    Ok(())
}

pub fn do_receive<T, I, D, E>(radio: &mut T, mut buff: &mut [u8], mut info: &mut I, mut delay: D, poll_interval: Duration) -> Result<usize, E> 
where
    T: Receive<Info=I, Error=E>,
    I: core::fmt::Debug,
    D: DelayMs<u32>,
{
    // Start receive mode
    radio.start_receive()?;

    loop {
        if radio.check_receive(true)? {
            let n = radio.get_received(&mut info, &mut buff)?;
            return Ok(n)
        }
        delay.delay_ms(poll_interval.as_millis() as u32);
    }
}

