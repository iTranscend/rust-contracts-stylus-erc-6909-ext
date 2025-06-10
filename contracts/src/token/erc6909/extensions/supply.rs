use alloc::{vec, vec::Vec};

use alloy_primitives::{FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    prelude::*,
    storage::{StorageMap, StorageU256},
};

use crate::{
    token::erc6909::{self, Erc6909, IErc6909, Error},
    utils::introspection::erc165::IErc165,
};

/// State of an [`Erc6909Supply`] contract.
#[storage]
pub struct Erc6909Supply {
    /// [`Erc6909`] contract.
    pub erc6909: Erc6909,
    /// Mapping from token id to token total_supply.
    pub(crate) total_supply: StorageMap<U256, StorageU256>,
}

#[public]
#[implements(IErc6909<Error = Error>, IErc6909Supply, IErc165)]
impl Erc6909Supply {}

#[interface_id]
pub trait IErc6909Supply: IErc165 {
    fn total_supply(&self, id: U256) -> U256;
}

#[public]
impl IErc165 for Erc6909Supply {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        <Self as IErc6909>::interface_id() == interface_id
    }
}

impl IErc6909Supply for Erc6909Supply {
    fn total_supply(&self, id: U256) -> U256 {
        // TODO: implement
        todo!()
    }
}

#[public]
impl IErc6909 for Erc6909Supply {
    type Error = erc6909::Error;
    
    // TODO: implement core interface
}

impl Erc6909Supply {}

#[cfg(test)]
mod tests {}
