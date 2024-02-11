//! Mock radio driver for application testing
//!
//! This provides a generic and specific mock implementation of the radio traits
//! to support network and application level testing.
//!
//! ## <https://github.com/rust-iot/radio-hal>
//! ## Copyright 2020-2022 Ryan Kurte

extern crate std;
use std::convert::Infallible;
use std::fmt::Debug;
use std::vec::Vec;

use log::debug;

use embedded_hal::delay::DelayNs;

use embedded_hal_mock::common::Generic;

use crate::{
    BasicInfo, Busy, Channel, Interrupts, Power, RadioState, Receive, ReceiveInfo, Rssi, State,
    Transmit,
};

/// Generic mock radio
///
/// Based on `embedded_hal_mock::common::Generic`
#[derive(Debug, Clone)]
pub struct Radio<
    St: Debug + Clone + PartialEq,
    Reg: Debug + Clone + PartialEq,
    Ch: Debug + Clone + PartialEq,
    Inf: Debug + Clone + PartialEq,
    Irq: Debug + Clone + PartialEq,
    E: Debug + Clone + PartialEq,
> {
    inner: Generic<Transaction<St, Reg, Ch, Inf, Irq, E>>,
}

impl<St, Reg, Ch, Inf, Irq, E> Radio<St, Reg, Ch, Inf, Irq, E>
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    pub fn new(expectations: &[Transaction<St, Reg, Ch, Inf, Irq, E>]) -> Self {
        let inner = Generic::new(expectations);

        Self { inner }
    }

    pub fn expect(&mut self, expectations: &[Transaction<St, Reg, Ch, Inf, Irq, E>]) {
        self.inner.expect(expectations);
    }

    pub fn next(&mut self) -> Option<Transaction<St, Reg, Ch, Inf, Irq, E>> {
        self.inner.next()
    }

    pub fn done(&mut self) {
        self.inner.done()
    }
}

/// Concrete mock radio using mock types
pub type MockRadio = Radio<MockState, u8, u8, BasicInfo, u8, MockError>;

/// MockState for use with mock radio
#[derive(Debug, Clone, PartialEq)]
pub enum MockState {
    Idle,
    Sleep,
    Receive,
    Receiving,
    Transmitting,
}

impl crate::RadioState for MockState {
    fn idle() -> Self {
        Self::Idle
    }

    fn sleep() -> Self {
        Self::Sleep
    }
}

/// MockError for use with mock radio
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum MockError {
    #[cfg_attr(feature = "thiserror", error("Timeout"))]
    Timeout,
}

/// Transactions describe interactions with a radio device
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction<St, Reg, Ch, Inf, Irq, E> {
    request: Request<St, Reg, Ch>,
    response: Response<St, Inf, Irq, E>,
}

impl<St, Reg, Ch, Inf, Irq, E> Transaction<St, Reg, Ch, Inf, Irq, E> {
    /// Set the radio state
    pub fn set_state(state: St, err: Option<E>) -> Self {
        Self {
            request: Request::SetState(state),
            response: err.into(),
        }
    }

    /// Get the radio state
    pub fn get_state(res: Result<St, E>) -> Self {
        Self {
            request: Request::GetState,
            response: res.map_or_else(Response::Err, Response::State),
        }
    }

    /// Check whether radio is currently busy
    pub fn is_busy(res: Result<bool, E>) -> Self {
        Self {
            request: Request::IsBusy,
            response: res.map_or_else(Response::Err, Response::Bool),
        }
    }

    /// Set a radio register
    pub fn set_register(reg: Reg, value: u8, err: Option<E>) -> Self {
        Self {
            request: Request::SetRegister(reg, value),
            response: err.into(),
        }
    }

    /// Get a radio register
    pub fn get_register(res: Result<u8, E>) -> Self {
        Self {
            request: Request::GetRegister,
            response: res.map_or_else(Response::Err, Response::Register),
        }
    }

    /// Set a radio channel
    pub fn set_channel(ch: Ch, err: Option<E>) -> Self {
        Self {
            request: Request::SetChannel(ch),
            response: err.into(),
        }
    }

    /// Set radio power
    pub fn set_power(power: i8, err: Option<E>) -> Self {
        Self {
            request: Request::SetPower(power),
            response: err.into(),
        }
    }

    /// Start radio transmission
    pub fn start_transmit(data: Vec<u8>, err: Option<E>) -> Self {
        Self {
            request: Request::StartTransmit(data),
            response: err.into(),
        }
    }

    /// Check for transmission completed
    pub fn check_transmit(res: Result<bool, E>) -> Self {
        Self {
            request: Request::CheckTransmit,
            response: res.map_or_else(Response::Err, Response::Bool),
        }
    }

    /// Start radio reception
    pub fn start_receive(err: Option<E>) -> Self {
        Self {
            request: Request::StartReceive,
            response: err.into(),
        }
    }

    /// Check for radio reception
    pub fn check_receive(restart: bool, res: Result<bool, E>) -> Self {
        Self {
            request: Request::CheckReceive(restart),
            response: res.map_or_else(Response::Err, Response::Bool),
        }
    }

    /// Fetch received data and information
    pub fn get_received(res: Result<(Vec<u8>, Inf), E>) -> Self {
        Self {
            request: Request::GetReceived,
            response: res.map_or_else(Response::Err, |(d, i)| Response::Received(d, i)),
        }
    }

    /// Fetch radio IRQs
    pub fn get_irq(clear: bool, res: Result<Irq, E>) -> Self {
        Self {
            request: Request::GetIrq(clear),
            response: res.map_or_else(Response::Err, Response::Irq),
        }
    }

    /// Poll for RSSI
    pub fn poll_rssi(res: Result<i16, E>) -> Self {
        Self {
            request: Request::PollRssi,
            response: res.map_or_else(Response::Err, Response::Rssi),
        }
    }

    /// Delay for a certain time
    pub fn delay_ns(ns: u32) -> Self {
        Self {
            request: Request::DelayNs(ns),
            response: Response::Ok,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Request<St, Reg, Ch> {
    SetState(St),
    GetState,
    IsBusy,

    SetRegister(Reg, u8),
    GetRegister,

    GetIrq(bool),
    PollRssi,

    SetChannel(Ch),
    SetPower(i8),

    StartTransmit(Vec<u8>),
    CheckTransmit,

    StartReceive,
    CheckReceive(bool),
    GetReceived,

    DelayNs(u32),
}

#[derive(Debug, Clone, PartialEq)]
enum Response<St, Inf, Irq, E> {
    Ok,
    State(St),
    Register(u8),
    Irq(Irq),
    Rssi(i16),
    Received(Vec<u8>, Inf),
    Bool(bool),
    Err(E),
}

impl<St, Inf, Irq, E> From<Option<E>> for Response<St, Inf, Irq, E> {
    fn from(e: Option<E>) -> Self {
        match e {
            Some(v) => Response::Err(v),
            None => Response::Ok,
        }
    }
}

impl<St, Reg, Ch, Inf, Irq, E> DelayNs for Radio<St, Reg, Ch, Inf, Irq, E>
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    fn delay_ns(&mut self, ns: u32) {
        let n = self.next().expect("no expectation for delay_ns call");

        assert_eq!(&n.request, &Request::DelayNs(ns));
    }
}

impl<St, Reg, Ch, Inf, Irq, E> State for Radio<St, Reg, Ch, Inf, Irq, E>
where
    St: RadioState + PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    type State = St;
    type Error = E;

    fn set_state(&mut self, state: Self::State) -> Result<(), Self::Error> {
        let n = self
            .next()
            .expect("no expectation for State::set_state call");

        assert_eq!(&n.request, &Request::SetState(state.clone()));

        let res = match &n.response {
            Response::Ok => Ok(()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        };

        debug!("Set state: {:?}: {:?}", state, res);

        res
    }

    fn get_state(&mut self) -> Result<Self::State, Self::Error> {
        let n = self
            .next()
            .expect("no expectation for State::get_state call");

        assert_eq!(&n.request, &Request::GetState);

        let res = match &n.response {
            Response::Err(e) => Err(e.clone()),
            Response::State(s) => Ok(s.clone()),
            _ => unreachable!(),
        };

        debug!("Get state {:?}", res);

        res
    }
}

impl<St, Reg, Ch, Inf, Irq, E> Busy for Radio<St, Reg, Ch, Inf, Irq, E>
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    type Error = E;

    fn is_busy(&mut self) -> Result<bool, Self::Error> {
        let n = self.next().expect("no expectation for is_busy call");

        assert_eq!(&n.request, &Request::IsBusy);

        let res = match &n.response {
            Response::Err(e) => Err(e.clone()),
            Response::Bool(s) => Ok(s.clone()),
            _ => unreachable!(),
        };

        debug!("Is busy {:?}", res);

        res
    }
}

impl<St, Reg, Ch, Inf, Irq, E> Channel for Radio<St, Reg, Ch, Inf, Irq, E>
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    type Channel = Ch;
    type Error = E;

    fn set_channel(&mut self, channel: &Self::Channel) -> Result<(), Self::Error> {
        debug!("Set channel {:?}", channel);

        let n = self
            .next()
            .expect("no expectation for State::set_channel call");

        assert_eq!(&n.request, &Request::SetChannel(channel.clone()));

        match &n.response {
            Response::Ok => Ok(()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
    }
}

impl<St, Reg, Ch, Inf, Irq, E> Power for Radio<St, Reg, Ch, Inf, Irq, E>
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    type Error = E;

    fn set_power(&mut self, power: i8) -> Result<(), Self::Error> {
        debug!("Set power {:?}", power);

        let n = self
            .next()
            .expect("no expectation for Power::set_power call");

        assert_eq!(&n.request, &Request::SetPower(power));

        match &n.response {
            Response::Ok => Ok(()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
    }
}

impl<St, Reg, Ch, Inf, Irq, E> Rssi for Radio<St, Reg, Ch, Inf, Irq, E>
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    type Error = E;

    fn poll_rssi(&mut self) -> Result<i16, Self::Error> {
        let n = self
            .next()
            .expect("no expectation for Rssi::poll_rssi call");

        assert_eq!(&n.request, &Request::PollRssi);

        let res = match &n.response {
            Response::Rssi(v) => Ok(v.clone()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        };

        debug!("Poll RSSI {:?}", res);

        res
    }
}

impl<St, Reg, Ch, Inf, Irq, E> Interrupts for Radio<St, Reg, Ch, Inf, Irq, E>
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    type Error = E;
    type Irq = Irq;

    fn get_interrupts(&mut self, clear: bool) -> Result<Self::Irq, Self::Error> {
        let n = self
            .next()
            .expect("no expectation for Transmit::check_transmit call");

        assert_eq!(&n.request, &Request::GetIrq(clear));

        let res = match &n.response {
            Response::Irq(v) => Ok(v.clone()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        };

        debug!("Get Interrupts {:?}", res);

        res
    }
}

impl<St, Reg, Ch, Inf, Irq, E> Transmit for Radio<St, Reg, Ch, Inf, Irq, E>
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    type Error = E;

    fn start_transmit(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let n = self
            .next()
            .expect("no expectation for Transmit::start_transmit call");

        assert_eq!(&n.request, &Request::StartTransmit(data.to_vec()));

        let res = match &n.response {
            Response::Ok => Ok(()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        };

        debug!("Start transmit {:?}: {:?}", data, res);

        res
    }

    fn check_transmit(&mut self) -> Result<bool, Self::Error> {
        let n = self
            .next()
            .expect("no expectation for Transmit::check_transmit call");

        assert_eq!(&n.request, &Request::CheckTransmit);

        let res = match &n.response {
            Response::Bool(v) => Ok(*v),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        };

        debug!("Check transmit {:?}", res);

        res
    }
}

impl<St, Reg, Ch, Inf, Irq, E> Receive for Radio<St, Reg, Ch, Inf, Irq, E>
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: ReceiveInfo + PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    type Info = Inf;
    type Error = E;

    fn start_receive(&mut self) -> Result<(), Self::Error> {
        let n = self
            .next()
            .expect("no expectation for Receive::start_receive call");

        assert_eq!(&n.request, &Request::StartReceive);

        let res = match &n.response {
            Response::Ok => Ok(()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        };

        debug!("Start receive {:?}", res);

        res
    }

    fn check_receive(&mut self, restart: bool) -> Result<bool, Self::Error> {
        let n = self
            .next()
            .expect("no expectation for Receive::check_receive call");

        assert_eq!(&n.request, &Request::CheckReceive(restart));

        let res = match &n.response {
            Response::Bool(v) => Ok(*v),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        };

        debug!("Check receive {:?}", res);

        res
    }

    fn get_received(&mut self, buff: &mut [u8]) -> Result<(usize, Self::Info), Self::Error> {
        let n = self
            .next()
            .expect("no expectation for Receive::get_received call");

        assert_eq!(&n.request, &Request::GetReceived);

        let res = match &n.response {
            Response::Received(d, i) => {
                buff[..d.len()].copy_from_slice(&d);

                Ok((d.len(), i.clone()))
            }
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        };

        debug!("Get received {:?}", res);

        res
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;

    #[test]
    fn test_radio_mock_set_state() {
        let mut radio = MockRadio::new(&[Transaction::set_state(MockState::Idle, None)]);

        radio.set_state(MockState::Idle).unwrap();

        radio.done();
    }

    #[test]
    #[should_panic]
    fn test_radio_mock_set_incorrect_state() {
        let mut radio = MockRadio::new(&[Transaction::set_state(MockState::Idle, None)]);

        radio.set_state(MockState::Sleep).unwrap();

        radio.done();
    }

    #[test]
    fn test_radio_mock_get_state() {
        let mut radio = MockRadio::new(&[Transaction::get_state(Ok(MockState::Idle))]);

        let res = radio.get_state().unwrap();
        assert_eq!(res, MockState::Idle);

        radio.done();
    }

    #[test]
    fn test_radio_mock_set_channel() {
        let mut radio = MockRadio::new(&[Transaction::set_channel(10, None)]);

        let _res = radio.set_channel(&10).unwrap();

        radio.done();
    }

    #[test]
    fn test_radio_mock_set_power() {
        let mut radio = MockRadio::new(&[Transaction::set_power(10, None)]);

        let _res = radio.set_power(10).unwrap();

        radio.done();
    }

    #[test]
    fn test_radio_mock_start_transmit() {
        let mut radio =
            MockRadio::new(&[Transaction::start_transmit(vec![0xaa, 0xbb, 0xcc], None)]);

        let _res = radio.start_transmit(&[0xaa, 0xbb, 0xcc]).unwrap();

        radio.done();
    }

    #[test]
    fn test_radio_mock_check_transmit() {
        let mut radio = MockRadio::new(&[
            Transaction::check_transmit(Ok(false)),
            Transaction::check_transmit(Ok(true)),
        ]);

        let res = radio.check_transmit().unwrap();
        assert_eq!(false, res);

        let res = radio.check_transmit().unwrap();
        assert_eq!(true, res);

        radio.done();
    }

    #[test]
    fn test_radio_mock_start_receive() {
        let mut radio = MockRadio::new(&[Transaction::start_receive(None)]);

        let _res = radio.start_receive().unwrap();

        radio.done();
    }

    #[test]
    fn test_radio_mock_check_receive() {
        let mut radio = MockRadio::new(&[
            Transaction::check_receive(true, Ok(false)),
            Transaction::check_receive(true, Ok(true)),
        ]);

        let res = radio.check_receive(true).unwrap();
        assert_eq!(false, res);

        let res = radio.check_receive(true).unwrap();
        assert_eq!(true, res);

        radio.done();
    }

    #[test]
    fn test_radio_mock_get_received() {
        let mut radio = MockRadio::new(&[Transaction::get_received(Ok((
            vec![0xaa, 0xbb],
            BasicInfo::new(10, 12),
        )))]);

        let mut buff = vec![0u8; 3];

        let (n, _i) = radio.get_received(&mut buff).unwrap();

        assert_eq!(2, n);
        assert_eq!(&buff[..2], &[0xaa, 0xbb]);

        radio.done();
    }
}
