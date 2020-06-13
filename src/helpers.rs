//! Provides common helpers for implementing radio utilities
//! 
//! ## https://github.com/ryankurte/rust-radio
//! ## Copyright 2020 Ryan Kurte

use structopt::StructOpt;
use humantime::{Duration as HumanDuration};

use embedded_hal::blocking::delay::DelayUs;

extern crate std;
use std::prelude::v1::*;

use std::time::{SystemTime};
use std::fs::{File, OpenOptions};
use std::ffi::CString;
use std::string::String;

use libc::{self};

use pcap_file::{PcapWriter, DataLink, pcap::PcapHeader};

use byteorder::{NetworkEndian, ByteOrder};

use crate::{Transmit, Receive, ReceiveInfo, Power, Rssi};
use crate::blocking::*;

/// Basic operations supported by the helpers package
#[derive(Clone, StructOpt, PartialEq, Debug)]
pub enum Operation {
    #[structopt(name="tx")]
    /// Transmit a packet
    Transmit(TransmitOptions),

    #[structopt(name="rx")]
    /// Receive a packet
    Receive(ReceiveOptions),

    #[structopt(name="rssi")]
    /// Poll RSSI on the configured channel
    Rssi(RssiOptions),

    #[structopt(name="echo")]
    /// Echo back received messages
    Echo(EchoOptions),
}

pub fn do_operation<T, I, E>(radio: &mut T, operation: Operation) -> Result<(), E> 
where
    T: Transmit<Error=E> + Power<Error=E> + Receive<Info=I, Error=E>  + Rssi<Error=E> + Power<Error=E> + DelayUs<u32>,
    I: ReceiveInfo + Default + std::fmt::Debug,
    E: std::fmt::Debug,
{
    let mut buff = [0u8; 1024];

    // TODO: the rest
    match operation {
        Operation::Transmit(options) => {
            do_transmit(radio, options)
                .expect("Transmit error")
        },
        Operation::Receive(options) => {
            let mut info = I::default();

            do_receive(radio, &mut buff, &mut info, options)
                .expect("Receive error");
        },
        Operation::Echo(options) => {
            let mut info = I::default();

            do_echo(radio, &mut buff, &mut info, options)
                .expect("Echo error");
        }
        Operation::Rssi(options) => {
            do_rssi(radio, options)
                .expect("RSSI error");
        },
        //_ => warn!("unsuppored command: {:?}", opts.command),
    }
    
    Ok(())
}

/// Configuration for Transmit operation
#[derive(Clone, StructOpt, PartialEq, Debug)]
pub struct TransmitOptions {
    /// Data to be transmitted
    #[structopt(long)]
    pub data: Vec<u8>,

    /// Power in dBm (range -18dBm to 13dBm)
    #[structopt(long)]
    pub power: Option<i8>,

    /// Specify period for repeated transmission
    #[structopt(long)]
    pub period: Option<HumanDuration>,

    #[structopt(flatten)]
    pub blocking_options: BlockingOptions,
}

pub fn do_transmit<T, E>(radio: &mut T, options: TransmitOptions) -> Result<(), BlockingError<E>> 
where
    T: Transmit<Error=E> + Power<Error=E> + DelayUs<u32>,
    E: core::fmt::Debug,
{
    // Set output power if specified
    if let Some(p) = options.power {
        radio.set_power(p)?;
    }

    loop {
        // Transmit packet
        radio.do_transmit(&options.data, options.blocking_options.clone())?;

        // Delay for repeated transmission or exit
        match &options.period {
            Some(p) => radio.delay_us(p.as_micros() as u32),
            None => break,
        }
    }

    Ok(())
}

/// Configuration for Receive operation
#[derive(Clone, StructOpt, PartialEq, Debug)]
pub struct ReceiveOptions {
    /// Run continuously
    #[structopt(long = "continuous")]
    pub continuous: bool,

    #[structopt(flatten)]
    pub pcap_options: PcapOptions,

    #[structopt(flatten)]
    pub blocking_options: BlockingOptions,
}

#[derive(Clone, StructOpt, PartialEq, Debug)]

pub struct PcapOptions {
    /// Create and write capture output to a PCAP file
    #[structopt(long, group="1")]
    pub pcap_file: Option<String>,

    /// Create and write to a unix pipe for connection to wireshark
    #[structopt(long, group="1")]
    pub pcap_pipe: Option<String>,
}

impl PcapOptions {
    pub fn open(&self) -> Result<Option<PcapWriter<File>>, std::io::Error> {

        // Open file or pipe if specified
        let pcap_file = match (&self.pcap_file, &self.pcap_pipe) {
            // Open as file
            (Some(file), None) => {
                let f = File::create(file)?;
                Some(f)
            },
            // Open as pipe
            #[cfg(target_family="unix")]
            (None, Some(pipe)) => {
                // Ensure file doesn't already exist
                let _ = std::fs::remove_file(pipe);
    
                // Create pipe
                let n = CString::new(pipe.as_str()).unwrap();
                let status = unsafe { libc::mkfifo(n.as_ptr(), 0o644) };
    
                // Manual status code handling
                // TODO: return io::Error
                if status != 0 {
                    panic!("Error creating fifo: {}", status);
                }
    
                // Open pipe
                let f = OpenOptions::new()
                    .write(true)
                    .open(pipe)
                    .expect("Error opening PCAP pipe");
    
                Some(f)
            }

            (None, None) => None,
            
            _ => unimplemented!()
        };

        info!("pcap pipe open, awaiting connection");

        // Setup pcap writer and write header
        // (This is a blocking operation on pipes)
        let pcap_writer = match pcap_file {
            None => None,
            Some(f) => {
                // Setup pcap header
                let mut h = PcapHeader::default();
                h.datalink = DataLink::IEEE802_15_4;

                // Write header
                let w = PcapWriter::with_header(h, f).expect("Error writing to PCAP file");
                Some(w)
            }
        };

        Ok(pcap_writer)
    }
}

/// Receive from the radio using the provided configuration
pub fn do_receive<T, I, E>(radio: &mut T, mut buff: &mut [u8], mut info: &mut I, options: ReceiveOptions) -> Result<usize, E> 
where
    T: Receive<Info=I, Error=E> + DelayUs<u32>,
    I: std::fmt::Debug,
{
    // Create and open pcap file for writing
    let mut pcap_writer = options.pcap_options.open().expect("Error opening pcap file / pipe");

    // Start receive mode
    radio.start_receive()?;

    loop {
        if radio.check_receive(true)? {
            let n = radio.get_received(&mut info, &mut buff)?;

            match std::str::from_utf8(&buff[0..n as usize]) {
                Ok(s) => info!("Received: '{}' info: {:?}", s, info),
                Err(_) => info!("Received: '{:x?}' info: {:?}", &buff[0..n as usize], info),
            }

            if let Some(p) = &mut pcap_writer {
                let t = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                
                p.write(t.as_secs() as u32, t.as_nanos() as u32 % 1_000_000, &buff[0..n], n as u32).expect("Error writing pcap file");
            }
            
            if !options.continuous { 
                return Ok(n)
            }

            radio.start_receive()?;
        }

        radio.delay_us(options.blocking_options.poll_interval.as_micros() as u32);
    }
}

/// Configuration for RSSI operation
#[derive(Clone, StructOpt, PartialEq, Debug)]
pub struct RssiOptions {
    /// Specify period for RSSI polling
    #[structopt(long = "period", default_value="1s")]
    pub period: HumanDuration,

    /// Run continuously
    #[structopt(long = "continuous")]
    pub continuous: bool,
}

pub fn do_rssi<T, I, E>(radio: &mut T, options: RssiOptions) -> Result<(), E> 
where
    T: Receive<Info=I, Error=E> + Rssi<Error=E> + DelayUs<u32>,
    I: std::fmt::Debug,
{
    // Enter receive mode
    radio.start_receive()?;

    // Poll for RSSI
    loop {
        let rssi = radio.poll_rssi()?;

        info!("rssi: {}", rssi);

        radio.check_receive(true)?;

        radio.delay_us(options.period.as_micros() as u32);

        if !options.continuous {
            break
        }
    }

    Ok(())
}

/// Configuration for Echo operation
#[derive(Clone, StructOpt, PartialEq, Debug)]
pub struct EchoOptions {
    /// Run continuously
    #[structopt(long = "continuous")]
    pub continuous: bool,
    
    /// Power in dBm (range -18dBm to 13dBm)
    #[structopt(long = "power")]
    pub power: Option<i8>,

    /// Specify delay for response message
    #[structopt(long = "delay", default_value="100ms")]
    pub delay: HumanDuration,

    /// Append RSSI and LQI to repeated message
    #[structopt(long = "append-info")]
    pub append_info: bool,

    #[structopt(flatten)]
    pub blocking_options: BlockingOptions,
}


pub fn do_echo<T, I, E>(radio: &mut T, mut buff: &mut [u8], mut info: &mut I, options: EchoOptions) -> Result<usize, BlockingError<E>> 
where
    T: Receive<Info=I, Error=E> + Transmit<Error=E> + Power<Error=E> + DelayUs<u32>,
    I: ReceiveInfo + std::fmt::Debug,
    E: std::fmt::Debug,
{
     // Set output power if specified
    if let Some(p) = options.power {
        radio.set_power(p)?;
    }

    // Start receive mode
    radio.start_receive()?;

    loop {
        if radio.check_receive(true)? {
            // Fetch received packet
            let mut n = radio.get_received(&mut info, &mut buff)?;

            // Parse out string if possible, otherwise print hex
            match std::str::from_utf8(&buff[0..n as usize]) {
                Ok(s) => info!("Received: '{}' info: {:?}", s, info),
                Err(_) => info!("Received: '{:02x?}' info: {:?}", &buff[0..n as usize], info),
            }

            // Append info if provided
            if options.append_info {
                NetworkEndian::write_i16(&mut buff[n..], info.rssi());
                n += 2;
            }

            // Wait for turnaround delay
            radio.delay_us(options.delay.as_micros() as u32);

            // Transmit respobnse
            radio.do_transmit(&buff[..n], options.blocking_options.clone())?;
            
            // Exit if non-continuous
            if !options.continuous { return Ok(n) }
        }

        // Wait for poll delay
        radio.delay_us(options.blocking_options.poll_interval.as_micros() as u32);
    }
}
