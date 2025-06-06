use alloc::{vec, vec::Vec};

use alloy_primitives::FixedBytes;
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::prelude::*;

use crate::utils::introspection::erc165::IErc165;

pub mod extensions;

mod sol {
    use alloy_sol_macro::sol;

    sol! {
        event Transfer();
        // TODO: fully define events
    }

    sol! {
        error InsufficientBalance();
        // TODO: fully define errors
    }
}

#[derive(SolidityError, Debug)]
pub enum Error {
    // TODO: Define errors
}

#[storage]
pub struct Erc6909 {
    // TODO: define storage maps for balances, operatorApprovals & allowances.
}

#[interface_id]
pub trait IErc6909: IErc165 {
    type Error: Into<alloc::vec::Vec<u8>>;

    // TODO: define core interface methods
    // transfer()
    // transfer_from
    // approve
    // set_operator
}

#[public]
#[implements(IErc6909<Error = Error>, IErc165)]
impl Erc6909 {}

impl IErc6909 for Erc6909 {
    type Error = Error;

    // TODO: implement core interface methods
    // transfer()
    // transfer_from
    // approve
    // set_operator
}

impl IErc165 for Erc6909 {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        <Self as IErc6909>::interface_id() == interface_id
    }
}

impl Erc6909 {
    // TODO: define private methods
    // _transfer
    // _burn
    // _update
    // _approve
    // _setOperator
    // _spendAllowance
}

#[cfg(test)]
mod tests {}
