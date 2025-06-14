//! Extension of ERC-6909 that adds tracking of total supply per token id.

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    msg,
    prelude::*,
    storage::{StorageMap, StorageU256},
};

use crate::{
    token::erc6909::{self, Erc6909, Error, IErc6909},
    utils::{
        introspection::erc165::IErc165,
        math::storage::{AddAssignChecked, SubAssignUnchecked},
    },
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

/// Required interface of a [`Erc6909Supply`] contract.
#[interface_id]
pub trait IErc6909Supply: IErc165 {
    /// Total amount of tokens with a given id.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id as a number.
    fn total_supply(&self, id: U256) -> U256;
}

#[public]
impl IErc165 for Erc6909Supply {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        <Self as IErc6909>::interface_id() == interface_id
            || self.erc6909.supports_interface(interface_id)
            || <Self as IErc165>::interface_id() == interface_id
    }
}

#[public]
impl IErc6909Supply for Erc6909Supply {
    fn total_supply(&self, id: U256) -> U256 {
        self.total_supply.get(id)
    }
}

#[public]
impl IErc6909 for Erc6909Supply {
    type Error = erc6909::Error;

    fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        let sender = msg::sender();
        self._transfer(sender, receiver, id, amount)
    }

    fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self._transfer(sender, receiver, id, amount)
    }

    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909.approve(spender, id, amount)
    }

    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error> {
        self.erc6909.set_operator(spender, approved)
    }

    fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.erc6909.balance_of(owner, id)
    }

    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256 {
        self.erc6909.allowance(owner, spender, id)
    }

    fn is_operator(&self, owner: Address, spender: Address) -> bool {
        self.erc6909.is_operator(owner, spender)
    }
}

impl Erc6909Supply {
    /// Creates an `amount` of tokens of type `id`, and assigns
    /// them to `to`.
    ///
    /// Re-export of [`Erc6909::_mint`].
    pub fn _mint(
        &mut self,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), erc6909::Error> {
        self._do_mint(to, vec![id], vec![amount])
    }

    /// Destroys an `amount` of tokens of type `id` from `from`.
    ///
    /// Re-export of [`Erc6909::_burn`].
    #[allow(clippy::missing_errors_doc)]
    pub fn _burn(
        &mut self,
        from: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), erc6909::Error> {
        self._do_burn(from, vec![id], vec![amount])
    }
}

impl Erc6909Supply {
    fn _do_mint(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
    ) -> Result<(), erc6909::Error> {
        if to.is_zero() {
            return Err(erc6909::Error::InvalidReceiver(
                erc6909::ERC6909InvalidReceiver { receiver: to },
            ));
        }

        self._update(Address::ZERO, to, ids.clone(), amounts.clone())?;

        Ok(())
    }

    fn _do_burn(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
    ) -> Result<(), erc6909::Error> {
        if from.is_zero() {
            return Err(erc6909::Error::InvalidSender(
                erc6909::ERC6909InvalidSender { sender: from },
            ));
        }

        self._update(from, Address::ZERO, ids, amounts)?;

        Ok(())
    }

    /// Extended version of [`Erc6909::_update`] that updates the supply of
    /// tokens.

    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token id.
    /// * `amounts` - Array of all amount of tokens to be supplied.
    ///
    /// # Errors
    ///
    /// * [`erc6909::Error::InvalidArrayLength`] - If length of `ids` is not
    ///   equal to length of `amounts`.
    /// * [`erc6909::Error::InsufficientBalance`] - If `amount` is greater than
    ///   the balance of the `from` account.
    ///
    /// # Events
    ///
    /// * [`erc6909::TransferSingle`] - If the arrays contain one element.
    /// * [`erc6909::TransferBatch`] - If the arrays contain more than one
    ///   element.
    ///
    /// # Panics
    ///
    /// * If updated balance and/or supply exceeds [`U256::MAX`], may happen
    ///   during the `mint` operation.
    fn _update(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
    ) -> Result<(), erc6909::Error> {
        self.erc6909._update(from, to, ids.clone(), amounts.clone())?;

        if from.is_zero() {
            for (&token_id, &amount) in ids.iter().zip(amounts.iter()) {
                self.total_supply.setter(token_id).add_assign_checked(
                    amount,
                    "should not exceed `U256::MAX` for `total_supply`",
                );
            }
        }

        if to.is_zero() {
            for (token_id, &amount) in ids.into_iter().zip(amounts.iter()) {
                self.total_supply.setter(token_id).sub_assign_unchecked(amount);
            }
        }

        Ok(())
    }

    fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, erc6909::Error> {
        if from.is_zero() {
            return Err(Error::InvalidSender(erc6909::ERC6909InvalidSender {
                sender: from,
            }));
        }
        if to.is_zero() {
            return Err(Error::InvalidReceiver(
                erc6909::ERC6909InvalidReceiver { receiver: to },
            ));
        }
        self._update(from, to, vec![id], vec![amount])?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {}
