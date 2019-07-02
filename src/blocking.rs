
extern crate std;
use blocking::std::*;
use blocking::std::time::Duration;

use crate::{Transmit, Receive, Power};

/// Blocking transmit function that uses thread_sleep internally to transmit a packet
pub fn do_transmit<T, E>(radio: &mut T, data: &[u8], power: Option<i8>, poll_interval: Duration) -> Result<(), E> 
where
    T: Transmit<Error=E> + Power<Error=E>
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
        std::thread::sleep(poll_interval);
    }

    Ok(())
}

pub fn do_receive<T, I, E>(radio: &mut T, mut buff: &mut [u8], mut info: &mut I, poll_interval: Duration) -> Result<usize, E> 
where
    T: Receive<Info=I, Error=E>,
    I: std::fmt::Debug,
{
    // Start receive mode
    radio.start_receive()?;

    loop {
        if radio.check_receive(true)? {
            let n = radio.get_received(&mut info, &mut buff)?;

            match std::str::from_utf8(&buff[0..n as usize]) {
                Ok(s) => info!("Received: '{}' info: {:?}", s, info),
                Err(_) => info!("Received: '{:?}' info: {:?}", &buff[0..n as usize], info),
            }
            
            return Ok(n)
        }
        std::thread::sleep(poll_interval);
    }
}

