//! Non-blocking APIs on top of the base radio traits
//! Note that this _requires_ (and will include) std
//! 
//! https://github.com/ryankurte/rust-radio
//! Copyright 2020 Ryan Kurte


use core::future::Future;
use core::marker::PhantomData;
use core::task::{Context, Poll, Waker};
use core::pin::Pin;

// std required for async-trait
extern crate std;
use std::boxed::Box;
use async_trait::async_trait;

extern crate async_std;

use embedded_hal::blocking::delay::DelayMs;
use crate::{Transmit, Receive, Power};

pub struct AsyncOptions {
    pub power: Option<i8>,
}

impl Default for AsyncOptions {
    fn default() -> Self {
        Self {            
            power: None,
        }
    }
}

/// Async transmit function implemented over `radio::Transmit` and `radio::Power` using the provided 
/// `AsyncOptions`
#[async_trait]
pub trait AsyncTransmit<E> {
    async fn async_transmit(&mut self, data: &[u8], tx_options: AsyncOptions) -> Result<(), E>;
}

#[async_trait]
impl <T, E: 'static> AsyncTransmit<E> for T
where
    T: Transmit<Error = E> + Power<Error = E> + DelayMs<u32> + Send,
    E: core::fmt::Debug + Send + Unpin,
{
    async fn async_transmit(&mut self, data: &[u8], tx_options: AsyncOptions) -> Result<(), E> {
        // Set output power if specified
        if let Some(p) = tx_options.power {
            self.set_power(p)?;
        }

        // Start transmission
        self.start_transmit(data)?;

        // Create transmit future
        let f: TransmitFuture<_, E> = TransmitFuture{radio: self, waker: None, _err: PhantomData};

        // Await on transmission
        let res = f.await?;

        // Return result
        Ok(res)
    }
}

pub struct TransmitFuture<'a, T, E> {
    radio: &'a mut T,
    waker: Option<Waker>,
    _err: PhantomData<E>,
}

impl <'a, T, E> Future for TransmitFuture<'a, T, E> 
where 
    T: Transmit<Error = E> + Power<Error = E> + DelayMs<u32> + Send,
    E: core::fmt::Debug + Send + Unpin,
{
    type Output = Result<(), E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let s = self.get_mut();

        // Check for completion
        if s.radio.check_transmit()? {
            return Poll::Ready(Ok(()))
        };
        
        // TODO: we don't _strictly_ need to wake every time?
        // but for now, we're going to
        cx.waker().clone().wake();

        // Store for later (probably not required with above)
        s.waker = Some(cx.waker().clone());

        // Indicate there is still work to be done
        Poll::Pending
    }
}

/// Async transmit function implemented over `radio::Transmit` and `radio::Power` using the provided 
/// `AsyncOptions`
#[async_trait]
pub trait AsyncReceive<I, E> {
    async fn async_receive(&mut self, info: &mut I, buff: &mut [u8], rx_options: AsyncOptions) -> Result<usize, E>;
}

#[async_trait]
impl <T, I, E: 'static> AsyncReceive<I, E> for T
where
    T: Receive<Error = E, Info = I> + DelayMs<u32> + Send,
    I: core::fmt::Debug + Send,
    E: core::fmt::Debug + Send + Unpin,
{
    async fn async_receive(&mut self, info: &mut I, buff: &mut [u8], _rx_options: AsyncOptions) -> Result<usize, E> {
        // Start receive mode
        self.start_receive()?;

        // Create receive future
        let f: ReceiveFuture<_, I, E> = ReceiveFuture {
            radio: self, info, buff, waker: None, _err: PhantomData
        };

        // Await completion
        let r = f.await?;

        // Return result
        Ok(r)
    }
}

pub struct ReceiveFuture<'a, T, I, E> {
    radio: &'a mut T,
    info: &'a mut I,
    buff: &'a mut [u8],
    waker: Option<Waker>,
    _err: PhantomData<E>,
}

impl <'a, T, I, E> Future for ReceiveFuture<'a, T, I, E> 
where 
    T: Receive<Error = E, Info = I> + DelayMs<u32> + Send,
    I: core::fmt::Debug + Send,
    E: core::fmt::Debug + Send + Unpin,
{
    type Output = Result<usize, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let s = self.get_mut();

        // Check for completion
        if s.radio.check_receive(true)? {
            // Retrieve data
            let n = s.radio.get_received(s.info, s.buff)?;

            return Poll::Ready(Ok(n));
        }

        // TODO: we don't _strictly_ need to wake every time?
        // but for now, we're going to
        cx.waker().clone().wake();

        // Store for later (probably not required with above)
        s.waker = Some(cx.waker().clone());

        // Indicate there is still work to be done
        Poll::Pending
    }
}
