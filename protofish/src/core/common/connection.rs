use std::sync::Arc;

use crate::{core::common::pmc::PMC, utp::protocol::UTP};

pub struct Connection<U>
where
    U: UTP,
{
    utp: Arc<U>,
    pub pmc: PMC<U::Stream>,
}

impl<U> Connection<U>
where
    U: UTP,
{
    pub fn new(utp: Arc<U>, pmc: PMC<U::Stream>) -> Self {
        Self { utp, pmc }
    }
}
