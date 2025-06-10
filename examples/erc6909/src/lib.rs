use openzeppelin_stylus::token::erc6909::{
    self, extensions::IErc6909Supply, Erc6909, IErc6909,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc6909Example {
    ecr6909: Erc6909,
}

#[public]
#[implements(IErc6909<Error = erc6909::Error>)]
impl Erc6909Example {}

#[public]
impl IErc6909 for Erc6909Example {
    type Error = erc6909::Error;

    // TODO: implement core interface
}

#[public]
impl IErc165 for Erc6909Example {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        self.erc6909.supports_interface(interface_id)
    }
}
