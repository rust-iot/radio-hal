//! Non-blocking (async/await) APIs on top of the base radio traits
//! Note that this _requires_ (and will include) std
//!
//! 
//! 
//! ## https://github.com/ryankurte/rust-radio
//! ## Copyright 2020 Ryan Kurte

use core::time::Duration;
use core::future::Future;
use core::marker::PhantomData;
use core::task::{Context, Poll};
use core::pin::Pin;

// std required for async-trait
extern crate std;
use std::boxed::Box;
use async_trait::async_trait;

extern crate async_std;
use async_std::{task};

use crate::{Transmit, Receive, Power};

pub struct AsyncOptions {
    pub power: Option<i8>,
    pub timeout: Option<Duration>,
    pub poll_period: Duration,
    // Use an async_std timer to wake after a specified duration
    // TODO: replace this with a callback so it can be generic over waker functions
    pub wake_fn: bool,
}

impl Default for AsyncOptions {
    fn default() -> Self {
        Self {            
            power: None,
            timeout: Some(Duration::from_millis(100)),
            poll_period: Duration::from_millis(10),
            wake_fn: false,
        }
    }
}

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
task::block_on(async {
    // Transmit using a future
    let res = radio.async_transmit(&[0xaa, 0xbb], AsyncOptions::default()).await;
    
    assert_eq!(res, Ok(()));
});

# radio.done();
```
"##)]
/// 
#[async_trait]
pub trait AsyncTransmit<E> {
    async fn async_transmit(&mut self, data: &[u8], tx_options: AsyncOptions) -> Result<(), AsyncError<E>> where E: 'async_trait;
}

#[async_trait]
impl <T, E> AsyncTransmit<E> for T
where
    T: Transmit<Error = E> + Power<Error = E> + Send,
    E: core::fmt::Debug + Send + Unpin,
{
    async fn async_transmit(&mut self, data: &[u8], tx_options: AsyncOptions) -> Result<(), AsyncError<E>> where E: 'async_trait,
    {
        // Set output power if specified
        if let Some(p) = tx_options.power {
            self.set_power(p)?;
        }

        // Start transmission
        self.start_transmit(data)?;

        // Create transmit future
        let f: TransmitFuture<_, E> = TransmitFuture{
            radio: self, 
            timeout: tx_options.timeout,
            period: tx_options.poll_period,
            wake_fn: tx_options.wake_fn,
            _err: PhantomData
        };

        // Await on transmission
        let res = f.await?;

        // Return result
        Ok(res)
    }
}

struct TransmitFuture<'a, T, E> {
    radio: &'a mut T,
    timeout: Option<Duration>,
    period: Duration,
    wake_fn: bool,
    _err: PhantomData<E>,
}


impl <'a, T, E> Future for TransmitFuture<'a, T, E> 
where 
    T: Transmit<Error = E> + Power<Error = E> + Send,
    E: core::fmt::Debug + Send + Unpin,
{
    type Output = Result<(), AsyncError<E>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let s = self.get_mut();
        let period = s.period.clone();

        // Check for completion
        if s.radio.check_transmit()? {
            return Poll::Ready(Ok(()))
        };
        
        trace!("remaining time: {:?}", s.timeout);

        // Check for timeout
        if let Some(v) = s.timeout.as_mut() {
            match v.checked_sub(period) {
                Some(t) => *v = t,
                None => return Poll::Ready(Err(AsyncError::Timeout))
            }
        }

        // Spawn task to re-execute waker
        let waker = cx.waker().clone();
        if !s.wake_fn {
            waker.wake();
        } else {
            task::spawn(async move {
                task::sleep(period).await;
                waker.wake();
            });
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
task::block_on(async {
    // Setup buffer and receive info
    let mut buff = [0u8; 128];
    let mut i = BasicInfo::new(0, 0);

    // Receive using a future
    let res = radio.async_receive(&mut i, &mut buff, AsyncOptions::default()).await;
    
    assert_eq!(res, Ok(data.len()));
    assert_eq!(&buff[..data.len()], &data);
});

# radio.done();
```
"##)]
/// 
#[async_trait]
pub trait AsyncReceive<I, E> {
    async fn async_receive(&mut self, info: &mut I, buff: &mut [u8], rx_options: AsyncOptions) -> Result<usize, AsyncError<E>> where E: 'async_trait;
}

#[async_trait]
impl <T, I, E> AsyncReceive<I, E> for T
where
    T: Receive<Error = E, Info = I> + Send,
    I: core::fmt::Debug + Send,
    E: core::fmt::Debug + Send + Unpin,
{
    async fn async_receive(&mut self, info: &mut I, buff: &mut [u8], rx_options: AsyncOptions) -> Result<usize, AsyncError<E>> where E: 'async_trait {
        // Start receive mode
        self.start_receive()?;

        // Create receive future
        let f: ReceiveFuture<_, I, E> = ReceiveFuture {
            radio: self, 
            info, 
            buff, 
            timeout: rx_options.timeout,
            period: rx_options.poll_period,
            wake_fn: rx_options.wake_fn,
            _err: PhantomData
        };

        // Await completion
        let r = f.await?;

        // Return result
        Ok(r)
    }
}

struct ReceiveFuture<'a, T, I, E> {
    radio: &'a mut T,
    info: &'a mut I,
    buff: &'a mut [u8],
    timeout: Option<Duration>,
    period: Duration,
    wake_fn: bool,
    _err: PhantomData<E>,
}

impl <'a, T, I, E> Future for ReceiveFuture<'a, T, I, E> 
where 
    T: Receive<Error = E, Info = I> + Send,
    I: core::fmt::Debug + Send,
    E: core::fmt::Debug + Send + Unpin,
{
    type Output = Result<usize, AsyncError<E>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let s = self.get_mut();
        let period = s.period.clone();

        // Check for completion
        if s.radio.check_receive(true)? {
            // Retrieve data
            let n = s.radio.get_received(s.info, s.buff)?;

            return Poll::Ready(Ok(n));
        }

        // Check for timeout
        if let Some(v) = s.timeout.as_mut() {
            match v.checked_sub(period) {
                Some(t) => *v = t,
                None => return Poll::Ready(Err(AsyncError::Timeout))
            }
        }

        // Spawn task to re-execute waker
        let waker = cx.waker().clone();
        if !s.wake_fn {
            waker.wake();
        } else {
            task::spawn(async move {
                task::sleep(period).await;
                waker.wake();
            });
        }

        // Indicate there is still work to be done
        Poll::Pending
    }
}
