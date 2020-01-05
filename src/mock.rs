//! Mock radio driver for application testing
//! 
//! This provides a generic and specific mock implementation of the radio traits
//! to support network and application level testing.
//! 
// https://github.com/ryankurte/rust-radio
// Copyright 2020 Ryan Kurte

extern crate std;
use std::vec::Vec;
use std::fmt::Debug;

extern crate embedded_hal_mock;
use embedded_hal_mock::common::Generic;

use crate::{State, Transmit, Receive, Power, Channel, Interrupts, BasicInfo};

/// Mock radio implementation
/// 
/// Based on `embedded_hal_mock::common::Generic`
pub type Radio<St, Reg, Ch, Inf, Irq, Err> = Generic<Transaction<St, Reg, Ch, Inf, Irq, Err>>;

pub type MockRadio = Radio<MockState, u8, u8, BasicInfo, u8, MockError>;

/// MockState for use with mock radio
#[derive(Debug, Clone, PartialEq)]
pub enum MockState {
    Idle,
    Sleep,
    Rx,
    Tx,
}


unsafe impl Send for MockState {}

/// MockError for use with mock radio
#[derive(Debug, Clone, PartialEq)]
pub enum MockError {
    Timeout,
}

unsafe impl Send for MockError {}

impl Unpin for MockError {}

/// Transactions describe interactions with a radio device
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction<St, Reg, Ch, Inf, Irq, E> {
    request: Request<St, Reg, Ch>,
    response: Response<St, Inf, Irq, E>,
}


unsafe impl <St, Reg, Ch, Inf, Irq, E>  Send for Transaction <St, Reg, Ch, Inf, Irq, E>  {}

impl <St, Reg, Ch, Inf, Irq, E> Transaction<St, Reg, Ch, Inf, Irq, E> {
    /// Set the radio state
    pub fn set_state(state: St, err: Option<E>) -> Self {
        Self{
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

    /// Set a radio register
    pub fn set_register(reg: Reg, value: u8, err: Option<E>) -> Self {
        Self{
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
        Self{
            request: Request::SetChannel(ch),
            response: err.into(),
        }
    }

    /// Set radio power
    pub fn set_power(power: i8, err: Option<E>) -> Self {
        Self{
            request: Request::SetPower(power),
            response: err.into(),
        }
    }

    /// Start radio transmission
    pub fn start_transmit(data: Vec<u8>, err: Option<E>) -> Self {
        Self{
            request: Request::StartTransmit(data),
            response: err.into(),
        }
    }

    /// Check for transmission completed
    pub fn check_transmit(res: Result<bool, E>) -> Self {
        Self{
            request: Request::CheckTransmit,
            response: res.map_or_else(Response::Err, Response::Bool),
        }
    }

    /// Start radio reception
    pub fn start_receive(err: Option<E>) -> Self {
        Self{
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
        Self{
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
}

#[derive(Debug, Clone, PartialEq)]
enum Request<St, Reg, Ch> {
    SetState(St),
    GetState,

    SetRegister(Reg, u8),
    GetRegister,

    GetIrq(bool),

    SetChannel(Ch),
    SetPower(i8),
    
    StartTransmit(Vec<u8>),
    CheckTransmit,

    StartReceive,
    CheckReceive(bool),
    GetReceived,
}

#[derive(Debug, Clone, PartialEq)]
enum Response<St, Inf, Irq, E> {
    Ok,
    State(St),
    Register(u8),
    Irq(Irq),
    Received(Vec<u8>, Inf),
    Bool(bool),
    Err(E),
}

impl <St, Inf, Irq, E> From<Option<E>> for Response<St, Inf, Irq, E> {
    fn from(e: Option<E>) -> Self {
        match e {
            Some(v) => Response::Err(v),
            None => Response::Ok,
        }
    }
}


impl <St, Reg, Ch, Inf, Irq, E> State for Radio<St, Reg, Ch, Inf, Irq, E> 
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    type State = St;
    type Error = E;

    fn set_state(&mut self, state: Self::State) -> Result<(), Self::Error> {
        let n = self.next().expect("no expectation for State::set_state call");

        assert_eq!(&n.request, &Request::SetState(state));

        match &n.response {
            Response::Ok => Ok(()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
    }

    fn get_state(&mut self) -> Result<Self::State, Self::Error> {
        let n = self.next().expect("no expectation for State::get_state call");

        assert_eq!(&n.request, &Request::GetState);

        match &n.response {
            Response::Err(e) => Err(e.clone()),
            Response::State(s) => Ok(s.clone()),
            _ => unreachable!(),
        }
    }

}

impl <St, Reg, Ch, Inf, Irq, E> Channel for Radio<St, Reg, Ch, Inf, Irq, E> 
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
        let n = self.next().expect("no expectation for State::set_channel call");

        assert_eq!(&n.request, &Request::SetChannel(channel.clone()));

        match &n.response {
            Response::Ok => Ok(()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
    }
}

impl <St, Reg, Ch, Inf, Irq, E> Power for Radio<St, Reg, Ch, Inf, Irq, E> 
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
        let n = self.next().expect("no expectation for Power::set_power call");

        assert_eq!(&n.request, &Request::SetPower(power));

        match &n.response {
            Response::Ok => Ok(()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
    }
}

impl <St, Reg, Ch, Inf, Irq, E> Interrupts for Radio<St, Reg, Ch, Inf, Irq, E> 
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
        let n = self.next().expect("no expectation for Transmit::check_transmit call");

        assert_eq!(&n.request, &Request::GetIrq(clear));

        match &n.response {
            Response::Irq(v) => Ok(v.clone()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
    }
}

impl <St, Reg, Ch, Inf, Irq, E> Transmit for Radio<St, Reg, Ch, Inf, Irq, E> 
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
        let n = self.next().expect("no expectation for Transmit::start_transmit call");

        assert_eq!(&n.request, &Request::StartTransmit(data.to_vec()));

        match &n.response {
            Response::Ok => Ok(()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
    }

    fn check_transmit(&mut self) -> Result<bool, Self::Error> {
        let n = self.next().expect("no expectation for Transmit::check_transmit call");

        assert_eq!(&n.request, &Request::CheckTransmit);

        match &n.response {
            Response::Bool(v) => Ok(*v),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
    }
}

impl <St, Reg, Ch, Inf, Irq, E> Receive for Radio<St, Reg, Ch, Inf, Irq, E> 
where
    St: PartialEq + Debug + Clone,
    Reg: PartialEq + Debug + Clone,
    Ch: PartialEq + Debug + Clone,
    Inf: PartialEq + Debug + Clone,
    Irq: PartialEq + Debug + Clone,
    E: PartialEq + Debug + Clone,
{
    type Info = Inf;
    type Error = E;

    fn start_receive(&mut self) -> Result<(), Self::Error> {
        let n = self.next().expect("no expectation for Receive::start_receive call");

        assert_eq!(&n.request, &Request::StartReceive);

        match &n.response {
            Response::Ok => Ok(()),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
    }

    fn check_receive(&mut self, restart: bool) -> Result<bool, Self::Error> {
        let n = self.next().expect("no expectation for Receive::check_receive call");

        assert_eq!(&n.request, &Request::CheckReceive(restart));

        match &n.response {
            Response::Bool(v) => Ok(*v),
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
    }

    fn get_received(&mut self, info: &mut Self::Info, buff: &mut [u8]) -> Result<usize, Self::Error> {
        let n = self.next().expect("no expectation for Receive::get_received call");

        assert_eq!(&n.request, &Request::GetReceived);

        match &n.response {
            Response::Received(d, i) => {
                &mut buff[..d.len()].copy_from_slice(&d);
                *info = i.clone();

                Ok(d.len())
            },
            Response::Err(e) => Err(e.clone()),
            _ => unreachable!(),
        }
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
        let mut radio = MockRadio::new(&[Transaction::start_transmit(vec![0xaa, 0xbb, 0xcc], None)]);

        let _res = radio.start_transmit(&[0xaa, 0xbb, 0xcc]).unwrap();

        radio.done();
    }

    #[test]
    fn test_radio_mock_check_transmit() {
        let mut radio = MockRadio::new(&[Transaction::check_transmit(Ok(false)), Transaction::check_transmit(Ok(true))]);

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
        let mut radio = MockRadio::new(&[Transaction::check_receive(true, Ok(false)), Transaction::check_receive(true, Ok(true))]);

        let res = radio.check_receive(true, ).unwrap();
        assert_eq!(false, res);

        let res = radio.check_receive(true, ).unwrap();
        assert_eq!(true, res);

        radio.done();
    }

    #[test]
    fn test_radio_mock_get_received() {
        let mut radio = MockRadio::new(&[Transaction::get_received(Ok((vec![0xaa, 0xbb], BasicInfo::new(10, 12))))]);

        let mut buff = vec![0u8; 3];
        let mut info = BasicInfo::new(0, 0);

        let res = radio.get_received(&mut info, &mut buff).unwrap();

        assert_eq!(2, res);
        assert_eq!(&buff[..2], &[0xaa, 0xbb]);


        radio.done();
    }
}
