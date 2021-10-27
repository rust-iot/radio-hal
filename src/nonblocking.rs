//! Non-blocking (async/await) APIs on top of the base radio traits
//! Note that this _requires_ use of unstable `feature(generic_associated_types)`
//!
//! ## https://github.com/ryankurte/rust-radio
//! ## Copyright 2020 Ryan Kurte

use core::time::Duration;
use core::future::Future;
use core::marker::PhantomData;
use core::task::{Context, Poll};
use core::pin::Pin;

use crate::{Transmit, Receive, Power};

/// Options for async driver calls
pub struct AsyncOptions {
    /// Power option, for transmit operations
    pub power: Option<i8>,
    
    /// Timeout option for underlying radio operations
    #[deprecated(note = "Timeouts must (currently) be implemented outside this module")]
    pub timeout: Option<Duration>,
    
    /// Period for polling on operation status with custom wakers
    pub poll_period: Duration,
    
    /// Waker function to be called in the `Poll` method
    pub wake_fn: Option<&'static fn(cx: &mut Context, d: Duration)>,
}

impl Default for AsyncOptions {
    #[allow(deprecated)]
    fn default() -> Self {
        Self {            
            power: None,
            timeout: None,
            poll_period: Duration::from_millis(10),
            wake_fn: None,
        }
    }
}

/// AsyncError wraps radio errors and provides notification of timeouts
#[derive(Clone, Debug, PartialEq)]
pub enum AsyncError<E> {
    Inner(E),
    Timeout,
}

impl <E> From<E> for AsyncError<E> {
    fn from(e: E) -> Self {
        AsyncError::Inner(e)
    }
}

/// Async transmit function implemented over `radio::Transmit` and `radio::Power` using the provided `AsyncOptions`
/// 
#[cfg_attr(feature = "mock", doc = r##"
```
extern crate async_std;
use async_std::task;

# use radio::*;
# use radio::mock::*;
use radio::nonblocking::{AsyncTransmit, AsyncOptions};

# let mut radio = MockRadio::new(&[
#    Transaction::start_transmit(vec![0xaa, 0xbb], None),
#    Transaction::check_transmit(Ok(false)),
#    Transaction::check_transmit(Ok(true)),
# ]);
# 
let res = task::block_on(async {
    // Transmit using a future
    radio.async_transmit(&[0xaa, 0xbb], AsyncOptions::default())?.await
});

assert_eq!(res, Ok(()));

# radio.done();
```
"##)]

/// AsyncTransmit function provides an async implementation for transmitting packets 
pub trait AsyncTransmit<'a, P, E> {
    type Output: Future<Output=Result<(), AsyncError<E>>>;

    fn async_transmit(&'a mut self, data: &'a [u8], params: &'a P, tx_options: AsyncOptions) -> Result<Self::Output, E>;
}


/// Future object containing a radio for transmit operation
pub struct TransmitFuture<'a, T, P, E> {
    radio: &'a mut T,
    params: &'a P,
    options: AsyncOptions,
    _err: PhantomData<E>,
}

/// `AsyncTransmit` object for all `Transmit` devices
impl <'a, T, P, E> AsyncTransmit<'a, P, E> for T
where
    T: Transmit<P, Error = E> + Power<Error = E> + 'a,
    P: 'a,
    E: core::fmt::Debug + Unpin,
{
    type Output = TransmitFuture<'a, T, P, E>;

    fn async_transmit(&'a mut self, data: &'a [u8], params: &'a P, tx_options: AsyncOptions) -> Result<Self::Output, E>
    {
        // Set output power if specified
        if let Some(p) = tx_options.power {
            self.set_power(p)?;
        }

        // Start transmission
        self.start_transmit(data, params)?;

        // Create transmit future
        let f: TransmitFuture<_, P, E> = TransmitFuture{
            radio: self,
            params,
            options: tx_options,
            _err: PhantomData
        };

        Ok(f)
    }
}


impl <'a, T, P, E> Future for TransmitFuture<'a, T, P, E>
where 
    T: Transmit<P, Error = E> + Power<Error = E>,
    E: core::fmt::Debug + Unpin,
{
    type Output = Result<(), AsyncError<E>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let s = self.get_mut();
        let period = s.options.poll_period.clone();

        // Check for completion
        if s.radio.check_transmit()? {
            return Poll::Ready(Ok(()))
        };
        
        // Spawn task to re-execute waker
        if let Some(w) = s.options.wake_fn {
            w(cx, period);
        } else {
            cx.waker().clone().wake();
        }

        // Indicate there is still work to be done
        Poll::Pending
    }
}


/// Async transmit function implemented over `radio::Transmit` and `radio::Power` using the provided `AsyncOptions`
/// 
#[cfg_attr(feature = "mock", doc = r##"
```
extern crate async_std;
use async_std::task;

# use radio::*;
# use radio::mock::*;
use radio::nonblocking::{AsyncReceive, AsyncOptions};

let data = [0xaa, 0xbb];
let info = BasicInfo::new(-81, 0);

# let mut radio = MockRadio::new(&[
#    Transaction::start_receive(None),
#    Transaction::check_receive(true, Ok(false)),
#    Transaction::check_receive(true, Ok(true)),
#    Transaction::get_received(Ok((data.to_vec(), info.clone()))),
# ]);
# 

// Setup buffer and receive info
let mut buff = [0u8; 128];
let mut i = BasicInfo::new(0, 0);

let res = task::block_on(async {
    // Receive using a future
    radio.async_receive(&mut i, &mut buff, AsyncOptions::default())?.await
});

assert_eq!(res, Ok(data.len()));
assert_eq!(&buff[..data.len()], &data);

# radio.done();
```
"##)]

/// AsyncReceive trait support futures-based polling on receive
pub trait AsyncReceive<'a, P, I, E> {
    type Output: Future<Output=Result<usize, AsyncError<E>>>;

    fn async_receive(&'a mut self, params: &'a P, info: &'a mut I, buff: &'a mut [u8], rx_options: AsyncOptions) -> Result<Self::Output, E>;
}

/// Receive future wraps a radio and buffer to provide a pollable future for receiving packets
pub struct ReceiveFuture<'a, T, P, I, E> {
    radio: &'a mut T,
    params: &'a P,
    info: &'a mut I,
    buff: &'a mut [u8],
    options: AsyncOptions,
    _err: PhantomData<E>,
}


/// Generic implementation of `AsyncReceive` for all `Receive` capable radio devices
impl <'a, T, P, I, E> AsyncReceive<'a, P, I, E> for T
where
    T: Receive<P, Error = E, Info = I> + 'a,
    P: 'a,
    I: core::fmt::Debug + 'a,
    E: core::fmt::Debug + Unpin,
{
    type Output = ReceiveFuture<'a, T, P, I, E>;

    fn async_receive(&'a mut self, params: &'a P, info: &'a mut I, buff: &'a mut [u8], rx_options: AsyncOptions) -> Result<Self::Output, E> {
        // Start receive mode
        self.start_receive(params)?;

        // Create receive future
        let f: ReceiveFuture<_, P, I, E> = ReceiveFuture {
            radio: self,
            params,
            info, 
            buff, 
            options: rx_options,
            _err: PhantomData
        };

        Ok(f)
    }
}

impl <'a, T, P, I, E> Future for ReceiveFuture<'a, T, P, I, E>
where 
    T: Receive<P, Error = E, Info = I>,
    I: core::fmt::Debug,
    E: core::fmt::Debug + Unpin,
{
    type Output = Result<usize, AsyncError<E>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let s = self.get_mut();

        // Check for completion
        if s.radio.check_receive(true)? {
            // Retrieve data
            let n = s.radio.get_received(s.info, s.buff)?;

            return Poll::Ready(Ok(n));
        }

        // TODO: should timeouts be internal or external?

        // Execute wake function
        if let Some(w) = s.options.wake_fn {
            w(cx, s.options.poll_period)
        } else {
            cx.waker().clone().wake();
        }

        // Indicate there is still work to be done
        Poll::Pending
    }
}

/// Task waker using async_std task::spawn with a task::sleep.
/// Note that this cannot be relied on for accurate timing
#[cfg(feature="async-std")]
pub fn async_std_task_waker(cx: &mut Context, period: Duration) {
    let waker = cx.waker().clone();
    async_std::task::spawn(async move {
        async_std::task::sleep(period).await;
        waker.wake();
    });
}
