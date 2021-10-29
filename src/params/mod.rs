use core::fmt::Debug;

use crate::{BasicInfo, ReceiveInfo};

/// The parameters of types of packages
pub trait Param {
    /// Packet received info
    type Info: ReceiveInfo + Debug;
}

/// No parameters necessary, for `Transmit<Basic>` and `Receive<Basic>`
pub struct Basic;

impl Param for Basic {
    type Info = BasicInfo;
}
