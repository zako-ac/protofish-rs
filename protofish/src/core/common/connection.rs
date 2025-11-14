use crate::{
    core::common::{
        arbitrary::{ArbContext, make_arbitrary},
        pmc::PMC,
    },
    utp::UTP,
};

pub struct Connection<U>
where
    U: UTP,
{
    pub pmc: PMC<U::Stream>,
}

impl<U> Connection<U>
where
    U: UTP,
{
    pub fn new(pmc: PMC<U::Stream>) -> Self {
        Self { pmc }
    }

    pub fn new_arb(&self) -> ArbContext<U::Stream> {
        let ctx = self.pmc.create_context();
        make_arbitrary(ctx)
    }

    pub async fn next_arb(&self) -> Option<ArbContext<U::Stream>> {
        let ctx = self.pmc.next_context().await?;
        Some(make_arbitrary(ctx))
    }
}
