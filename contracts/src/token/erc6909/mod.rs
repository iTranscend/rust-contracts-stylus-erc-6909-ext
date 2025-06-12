//! Implementation of the ERC-6909 token standard.
use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
// pub mod extensions;
pub use sol::*;
use stylus_sdk::{
    evm, msg,
    prelude::*,
    storage::{StorageBool, StorageMap, StorageU256},
};

use crate::utils::{
    introspection::erc165::IErc165,
    math::storage::{AddAssignChecked, SubAssignUnchecked},
};

mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when a `caller` transfers an `amount` of token `id`
        /// from a `sender` to a receiver.
        ///
        /// * `caller` - Address of the initiator of the transfer.
        /// * `sender` - Address of the sender.
        /// * `receiver` - Address of the receiver.
        /// * `id` - Token id as a number.
        /// * `amount` - Amount of token transferred.
        #[derive(Debug)]
        event Transfer(
            address caller,
            address indexed sender,
            address indexed receiver,
            uint256 indexed id,
            uint256 amount,
        );

        /// Emitted when a token `owner` sets the `approved` status of
        /// a `spender`.
        ///
        /// * `owner` - Address of the owner of the token.
        /// * `spender` - Address of the spender.
        /// * `approved` - Approved status as a boolean.
        #[derive(Debug)]
        event OperatorSet(
            address indexed owner,
            address indexed spender,
            bool approved,
        );

        /// Emitted when a token `owner` has approved a `spender` to
        /// transfer an `amount` of a token `id` to be transferred
        /// on the owner’s behalf.
        ///
        /// * `owner` - Address of the owner of the token.
        /// * `spender` - Address of the spender.
        /// * `id` - Token id as a number.
        /// * `amount` - Amount of token approved to be transferred.
        #[derive(Debug)]
        event Approval(
            address indexed owner,
            address indexed spender,
            uint256 indexed id,
            uint256 amount,
        );
    }

    sol! {
        /// Thrown when `owner`` balance for `id`` is insufficient.
        ///
        /// * `owner` - Address of the owner of the token.
        /// * `id` - Token id as a number.
        #[derive(Debug)]
        error Erc6909InsufficientBalance(
            address sender,
            uint256 balance,
            uint256 needed,
            uint256 id,
        );

        /// Thrown when the spender does not have permission to
        /// spend the token.
        ///
        /// * `spender` - Address of the spender
        /// * `id` - Token id as a number
        #[derive(Debug)]
        error Erc6909InsufficientPermission(
            address spender,
            uint256 id
        );

        /// Thrown when a `spender`'s allowance for a token type
        /// of `id`` is insufficient.
        ///
        /// * `owner` - Address of the owner of the token.
        /// * `id` - Token id as a number.
        #[derive(Debug)]
        error Erc6909InsufficientAllowance(
            address spender,
            uint256 allowance,
            uint256 needed,
            uint256 id,
        );

        /// Indicates a failure with the token `sender`.
        /// Used in transfers.
        ///
        /// * `approver` - Approver of the token.
        #[derive(Debug)]
        error ERC6909InvalidApprover(address approver);

        /// Indicates a failure with the token `sender`.
        /// Used in transfers.
        ///
        /// * `sender` - Address whose tokens are being transferred.
        #[derive(Debug)]
        error ERC6909InvalidSender(address sender);

        /// Indicates a failure with the token `spender`.
        /// Used in transfers.
        ///
        /// * `spender` - Address attempting to spend tokens on behalf
        /// of another address.
        #[derive(Debug)]
        error ERC6909InvalidSpender(address spender);

        /// Indicates a failure with the token `receiver`.
        /// Used in transfers.
        ///
        /// * `receiver` - Address to which tokens are being transferred.
        #[derive(Debug)]
        error ERC6909InvalidReceiver(address receiver);
    }
}

/// An [`Erc6909`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an owner's token balance is insufficient
    InsufficientBalance(Erc6909InsufficientBalance),
    /// Indicates the spender does not have permission to spend the token.
    InsufficientPermission(Erc6909InsufficientPermission),
    /// Indicates a spender's token allowance is insufficient
    InsufficientAllowance(Erc6909InsufficientAllowance),
    /// Indicates the approver is invalid.
    InvalidApprover(ERC6909InvalidApprover),
    /// Indicates the sender is invalid.
    InvalidSender(ERC6909InvalidSender),
    /// Indicates the sender is invalid.
    InvalidSpender(ERC6909InvalidSpender),
    /// Indicates the receiver is invalid.
    InvalidReceiver(ERC6909InvalidReceiver),
}

/// State of an [`Erc6909`] token.
#[storage]
pub struct Erc6909 {
    /// Maps owner addresses to token balances
    pub(crate) balances: StorageMap<Address, StorageMap<U256, StorageU256>>,
    /// Maps owner addresses to operator approval statuses
    pub(crate) operator_approvals:
        StorageMap<Address, StorageMap<Address, StorageBool>>,
    ///Maps owner to a mapping of spender allowances for each token id.
    pub(crate) allowances:
        StorageMap<Address, StorageMap<Address, StorageMap<U256, StorageU256>>>,
}

/// Required interface of an [`Erc6909`] compliant contract.
#[interface_id]
pub trait IErc6909: IErc165 {
    /// The error type associated to this ERC-6909 trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Transfers `amount` tokens of token type `id` from caller to
    /// `receiver`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `receiver` - Address to which tokens are being transferred.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount of token transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `from` is zero address.
    /// * [`Error::InvalidReceiver`] - If `to` is zero address.
    /// * [`Error::InsufficientBalance`] - If `from` address's balaance is less
    ///   that `amount`.
    ///
    /// # Events
    ///
    /// * [`Transfer`] event.
    ///
    /// Returns a boolean value indicating success or failure.
    fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error>;

    /// Transfers `amount` tokens of token type `id` from `sender` to
    /// `receiver`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - Address whose tokens are being transferred.
    /// * `receiver` - Address to which tokens are being transferred.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount of token transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `from` is zero address.
    /// * [`Error::InvalidReceiver`] - If `to` is zero address.
    /// * [`Error::InsufficientBalance`] - If `from` address's balaance is less
    ///   that `amount`.
    /// * [`Error::InsufficientAllowance`] - If the caller does not have enough
    ///   allowance to spend `amount`
    ///
    /// # Events
    ///
    /// * [`Transfer`] event.
    ///
    /// Returns a boolean value indicating success or failure.
    fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error>;

    /// Approves an amount of an id to a spender.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `spender` - The address of the spender.
    /// * `id` - The id of the token.
    /// * `amount` - The amount of the token.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidApprover`] - If `owner` is zero address
    /// * [`Error::InvalidSpender`] - If `spender` is zero address
    ///
    /// # Events
    ///
    /// * [`Approval`] event.
    ///
    /// Returns a boolean value indicating success or failure.
    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error>;

    /// Sets or removes a spender as an operator for the caller.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `spender` - The address of the spender.
    /// * `approved` - The approval status.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidApprover`] - If `owner` is zero address
    /// * [`Error::InvalidSpender`] - If `spender` is zero address
    ///
    /// # Events
    ///
    /// * [`OperatorSet `] event.
    ///
    /// Returns a boolean value indicating success or failure.
    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error>;

    /// Returns the value of tokens of type `id` owned by `owner`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Address of the token's owner.
    /// * `id` - Token id as a number.
    fn balance_of(&self, owner: Address, id: U256) -> U256;

    /// Returns the value of tokens of type `id` owned by `owner`,
    /// that can be spent by `spender`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Address of the token's owner.
    /// * `spender` - Address of the spender.
    /// * `id` - Token id as a number.
    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256;

    /// Returns true if `spender` is approved as an operator
    /// for `owner`'s account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    /// * `spender` - Account to be checked.
    fn is_operator(&self, owner: Address, spender: Address) -> bool;
}

#[public]
#[implements(IErc6909<Error = Error>, IErc165)]
impl Erc6909 {}

#[public]
impl IErc6909 for Erc6909 {
    type Error = Error;

    fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        let sender = msg::sender();
        self._transfer(sender, receiver, id, amount)?;
        Ok(true)
    }

    fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        let caller = msg::sender();

        if !self.is_operator(sender, caller) && sender != caller {
            self._spend_allowance(sender, caller, id, amount)?;
        }

        self._transfer(sender, receiver, id, amount)?;
        Ok(true)
    }

    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        let caller = msg::sender();
        self._approve(caller, spender, id, amount)?;
        Ok(true)
    }

    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error> {
        let caller = msg::sender();
        self._set_operator(caller, spender, approved)?;
        Ok(true)
    }

    fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.balances.get(owner).get(id)
    }

    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256 {
        self.allowances.get(owner).get(spender).get(id)
    }

    fn is_operator(&self, owner: Address, spender: Address) -> bool {
        self.operator_approvals.get(owner).get(spender)
    }
}

#[public]
impl IErc165 for Erc6909 {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        <Self as IErc6909>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
    }
}

impl Erc6909 {
    /// Moves `amount` of token `id` from `from` to `to` without checking for
    /// approvals. This function verifies that neither the sender nor the
    /// receiver are address(0), which means it cannot mint or burn tokens.
    /// Relies on the `_update` function.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - Address whose tokens are being transferred.
    /// * `receiver` - Address to which tokens are being transferred.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount of token transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `from` is zero address.
    /// * [`Error::InvalidReceiver`] - If `to` is zero address.
    ///
    /// # Events
    ///
    /// * [`Transfer`] event.
    fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        if from.is_zero() {
            return Err(Error::InvalidSender(ERC6909InvalidSender {
                sender: from,
            }));
        }
        if to.is_zero() {
            return Err(Error::InvalidReceiver(ERC6909InvalidReceiver {
                receiver: to,
            }));
        }
        self._update(from, to, id, amount)?;
        Ok(())
    }

    /// Transfers `amount` of token `id` from `from` to `to`
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Address whose tokens are being transferred.
    /// * `to` - Address to which tokens are being transferred.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount of token transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InsufficientBalance`] - If `from` address's balaance is less
    ///   that `amount`.
    ///
    /// # Events
    ///
    /// * [`Transfer`] event.
    fn _update(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        let caller = msg::sender();

        if !from.is_zero() {
            let from_balance = self.balance_of(from, id);
            if from_balance < amount {
                return Err(Error::InsufficientBalance(
                    Erc6909InsufficientBalance {
                        sender: from,
                        balance: from_balance,
                        needed: amount,
                        id,
                    },
                ));
            }
            self.balances.setter(from).setter(id).sub_assign_unchecked(amount);
        }
        if !to.is_zero() {
            self.balances.setter(to).setter(id).add_assign_checked(
                amount,
                "should not exceed `U256::MAX` for `balances`",
            );
        }

        evm::log(Transfer { caller, sender: from, receiver: to, id, amount });

        Ok(())
    }

    /// Sets `amount` as the allowance of `spender` over the `owner`'s `id`
    /// tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Address of acccount whose tokens a `spender` is approved to
    ///   spend.
    /// * `spender` - Address of account that will be allowed to spend an
    ///   `amount` of `owner`'s tokens.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount of tokens `spender` is allowed to spend on behalf of
    ///   `owner`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidApprover`] - If `owner` is zero address
    /// * [`Error::InvalidSpender`] - If `spender` is zero address
    ///
    /// # Events
    ///
    /// * [`Approval`] event.
    fn _approve(
        &mut self,
        owner: Address,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        if owner.is_zero() {
            return Err(Error::InvalidApprover(ERC6909InvalidApprover {
                approver: owner,
            }));
        }
        if spender.is_zero() {
            return Err(Error::InvalidSpender(ERC6909InvalidSpender {
                spender,
            }));
        }

        self.allowances.setter(owner).setter(spender).setter(id).set(amount);
        evm::log(Approval { owner, spender, id, amount });

        Ok(())
    }

    /// Sets `amount` as the allowance of `spender` over the `owner`'s `id`
    /// tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Address of acccount whose tokens a `spender` is approved to
    ///   spend.
    /// * `spender` - Address of account that will be allowed to spend an
    ///   `amount` of `owner`'s tokens.
    /// * `approved` - Boolean status of whether an operator is approved or not
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidApprover`] - If `owner` is zero address
    /// * [`Error::InvalidSpender`] - If `spender` is zero address
    ///
    /// # Events
    ///
    /// * [`OperatorSet `] event.
    fn _set_operator(
        &mut self,
        owner: Address,
        spender: Address,
        approved: bool,
    ) -> Result<(), Error> {
        if owner.is_zero() {
            return Err(Error::InvalidApprover(ERC6909InvalidApprover {
                approver: owner,
            }));
        }
        if spender.is_zero() {
            return Err(Error::InvalidSpender(ERC6909InvalidSpender {
                spender,
            }));
        }

        self.operator_approvals.setter(owner).setter(spender).set(approved);
        evm::log(OperatorSet { owner, spender, approved });

        Ok(())
    }

    /// Updates `owner`'s allowance for `spender` based on spent `amount`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Address of acccount whose tokens a `spender` is attempting
    ///   to spend.
    /// * `spender` - Address of account is spending an `amount` of `owner`'s
    ///   tokens.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount of tokens `spender` is attempting to spend on behalf
    ///   of `owner`.
    ///
    /// # Errors
    ///
    /// * [`Error::InsufficientAllowance`] - If `spender` does not have enough
    ///   allowance to spend `amount`
    fn _spend_allowance(
        &mut self,
        owner: Address,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        let current_allowance = self.allowance(owner, spender, id);

        if amount > current_allowance {
            return Err(Error::InsufficientAllowance(
                Erc6909InsufficientAllowance {
                    spender,
                    allowance: current_allowance,
                    needed: current_allowance,
                    id,
                },
            ));
        }

        self.allowances
            .setter(owner)
            .setter(spender)
            .setter(id)
            .sub_assign_unchecked(amount);

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
