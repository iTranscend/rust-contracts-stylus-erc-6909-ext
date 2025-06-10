use openzeppelin_stylus::{
    token::erc6909::{
        self,
        extensions::{Erc6909Supply, IErc6909Supply},
        IErc6909,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc6909Example {
    ecr6909: Erc6909Supply,
}

#[public]
#[implements(IErc6909<Error = erc6909::Error>, IErc6909Supply, IErc165)]
impl Erc6909Example {}

#[public]
impl IErc6909 for Erc6909Example {
    type Error = erc6909::Error;

    // TODO: implement core interface
}

#[public]
impl IErc6909Supply for Erc6909Example {
    fn total_supply(&self, id: U256) -> U256 {
        todo!()
    }
}

#[public]
impl IErc165 for Erc6909Example {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        self.erc6909.supports_interface(interface_id)
    }
}
