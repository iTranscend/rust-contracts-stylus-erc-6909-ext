#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::result_large_err)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
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
struct Erc6909SupplyExample {
    erc6909_supply: Erc6909Supply,
}

#[public]
impl IErc6909 for Erc6909SupplyExample {
    type Error = erc6909::Error;

    fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909_supply.transfer(receiver, id, amount)
    }

    fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909_supply.transfer_from(sender, receiver, id, amount)
    }

    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909_supply.approve(spender, id, amount)
    }

    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error> {
        self.erc6909_supply.set_operator(spender, approved)
    }

    fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.erc6909_supply.balance_of(owner, id)
    }

    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256 {
        self.erc6909_supply.allowance(owner, spender, id)
    }

    fn is_operator(&self, owner: Address, spender: Address) -> bool {
        self.erc6909_supply.is_operator(owner, spender)
    }
}

#[public]
impl IErc6909Supply for Erc6909SupplyExample {
    fn total_supply(&self, id: U256) -> U256 {
        self.erc6909_supply.total_supply(id)
    }
}

#[public]
impl IErc165 for Erc6909SupplyExample {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        self.erc6909_supply.supports_interface(interface_id)
    }
}

#[public]
#[implements(IErc6909<Error = erc6909::Error>, IErc6909Supply, IErc165)]
impl Erc6909SupplyExample {
    fn mint(
        &mut self,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), <Erc6909SupplyExample as IErc6909>::Error> {
        self.erc6909_supply._mint(to, id, amount)
    }

    fn mint_batch(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
    ) -> Result<(), <Erc6909SupplyExample as IErc6909>::Error> {
        self.erc6909_supply._mint_batch(to, ids, amounts)
    }

    fn burn(
        &mut self,
        from: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), <Erc6909SupplyExample as IErc6909>::Error> {
        self.erc6909_supply._burn(from, id, amount)
    }

    fn burn_batch(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
    ) -> Result<(), <Erc6909SupplyExample as IErc6909>::Error> {
        self.erc6909_supply._burn_batch(from, ids, amounts)
    }
}
