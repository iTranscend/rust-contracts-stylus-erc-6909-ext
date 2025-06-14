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

        /// Emitted when `value` amount of tokens of type `id` are
        /// transferred from `from` to `to` by `caller`.
        #[derive(Debug)]
        event TransferSingle(
            address indexed caller,
            address indexed from,
            address indexed to,
            uint256 id,
            uint256 amount
        );

        /// Equivalent to multiple [`TransferSingle`] events, where `caller`
        /// `from` and `to` are the same for all transfers.
        #[derive(Debug)]
        event TransferBatch(
            address indexed caller,
            address indexed from,
            address indexed to,
            uint256[] ids,
            uint256[] amounts
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

        /// Indicates an array length mismatch between token ids and values in a
        /// batch operation.
        ///
        /// * `ids_length` - Length of the array of token identifiers.
        /// * `values_length` - Length of the array of token amounts.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC6909InvalidArrayLength(
            uint256 ids_length,
            uint256 values_length
        );
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
    /// Indicates an array length mismatch between token ids and values in a
    /// batch operation.
    InvalidArrayLength(ERC6909InvalidArrayLength),
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

/// Implementation of [`TopLevelStorage`]
unsafe impl TopLevelStorage for Erc6909 {}

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
        self._update(from, to, vec![id], vec![amount])?;
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
        ids: Vec<U256>,
        amounts: Vec<U256>,
    ) -> Result<(), Error> {
        Self::require_equal_arrays_length(&ids, &amounts)?;

        let caller = msg::sender();

        for (&token_id, &amount) in ids.iter().zip(amounts.iter()) {
            self._do_update(from, to, token_id, amount)?;
        }

        if ids.len() == 1 {
            let id = ids[0];
            let amount = amounts[0];
            evm::log(TransferSingle { caller, from, to, id, amount });
        } else {
            evm::log(TransferBatch { caller, from, to, ids, amounts });
        }
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

    /// Grants `spender` operator privileges over the `owner`'s account.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Address of acccount whose tokens a `spender` is approved to
    ///   spend.
    /// * `spender` - Address of account that will be allowed to spend an
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

    /// Creates an `amount` amount of tokens of type `id`, and assigns
    /// them to `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `id` - Token id.
    /// * `amount` - Amount of tokens to be minted.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`TransferSingle`].
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds [`U256::MAX`].
    pub fn _mint(
        &mut self,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        self._do_mint(to, vec![id], vec![amount])
    }

    /// Batched version of [`Self::_mint`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all tokens ids to be minted.
    /// * `amounts` - Array of all amounts of tokens to be minted.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] -  If `to` is [`Address::ZERO`].
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `amounts`.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds [`U256::MAX`].
    pub fn _mint_batch(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
    ) -> Result<(), Error> {
        self._do_mint(to, ids, amounts)
    }

    /// Destroys an `amount` of tokens of type `id` from `from`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to burn tokens from.
    /// * `id` - Token id to be burnt.
    /// * `amount` - Amount of tokens to be burnt.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `from` is the [`Address::ZERO`].
    /// * [`Error::InsufficientBalance`]  - If `amount` is greater than the
    ///   token 'id' balance of the `from` account.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`].
    pub fn _burn(
        &mut self,
        from: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        self._do_burn(from, vec![id], vec![amount])
    }

    /// Batched version of [`Self::_burn`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to burn tokens from.
    /// * `ids` - Array of all tokens ids to be burnt.
    /// * `amounts` - Array of all amounts of tokens to be burnt.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `from` is the [`Address::ZERO`].
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `amounts`.
    /// * [`Error::InsufficientBalance`] - If any of the `amounts` is greater
    ///   than the balance of the respective token from `tokens` of the `from`
    ///   account.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.
    pub fn _burn_batch(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
    ) -> Result<(), Error> {
        self._do_burn(from, ids, amounts)
    }
}

impl Erc6909 {
    /// Creates `amounts` of tokens specified by `ids`, and assigns
    /// them to `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token ids to be minted.
    /// * `amounts` - Array of all amounts of tokens to be minted.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
    /// * [`Error::InvalidArrayLength`] -  If length of `ids` is not equal to
    ///   length of `amounts`.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the array contain multiple elements.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds [`U256::MAX`].
    fn _do_mint(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(Error::InvalidReceiver(ERC6909InvalidReceiver {
                receiver: to,
            }));
        }

        self._update(Address::ZERO, to, ids, amounts)?;

        Ok(())
    }

    // Destroys `amounts` of tokens specified by `ids` from `from`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to burn tokens from.
    /// * `ids` - Array of all token ids to be burnt.
    /// * `amounts` - Array of all amount of tokens to be burnt.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `from` is the [`Address::ZERO`].
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `amounts`.
    /// * [`Error::InsufficientBalance`] - If any of the `amounts` is greater
    ///   than the balance of the respective token from `ids` of the `from`
    ///   account.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.
    fn _do_burn(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
    ) -> Result<(), Error> {
        if from.is_zero() {
            return Err(Error::InvalidSender(ERC6909InvalidSender {
                sender: from,
            }));
        }
        self._update(from, Address::ZERO, ids, amounts)?;
        Ok(())
    }

    /// Checks if `ids` array has same length as `values` array.
    ///
    /// # Arguments
    ///
    /// * `ids` - array of `ids`.
    /// * `values` - array of `values`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `values`.
    fn require_equal_arrays_length<T, U>(
        ids: &[T],
        values: &[U],
    ) -> Result<(), Error> {
        if ids.len() != values.len() {
            return Err(Error::InvalidArrayLength(ERC6909InvalidArrayLength {
                ids_length: U256::from(ids.len()),
                values_length: U256::from(values.len()),
            }));
        }
        Ok(())
    }

    /// Transfers a `amount` amount of `id` from `from` to
    /// `to`. Will mint (or burn) if `from` (or `to`) is the [`Address::ZERO`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `id` - Token id.
    /// * `amount` - Amount of tokens to be transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InsufficientBalance`] - If `amount` is greater than the
    ///   balance of the `from` account.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds [`U256::MAX`].
    fn _do_update(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
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
        // else {
        //     self.balances.setter(to).setter(id).add_assign_checked(
        //         amount,
        //         "should not exceed `U256::MAX` for `balances`",
        //     );
        // }
        if !to.is_zero() {
            self.balances.setter(to).setter(id).add_assign_checked(
                amount,
                "should not exceed `U256::MAX` for `balances`",
            );
        }
        // else {
        //     // Burn
        //     let from_balance = self.balance_of(from, id);
        //     if from_balance < amount {
        //         return Err(Error::InsufficientBalance(
        //             Erc6909InsufficientBalance {
        //                 sender: from,
        //                 balance: from_balance,
        //                 needed: amount,
        //                 id,
        //             },
        //         ));
        //     }
        //     self.balances.setter(from).setter(id).
        // sub_assign_unchecked(amount); }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{fixed_bytes, uint, Address, FixedBytes, U256};
    use motsu::prelude::*;
    use stylus_sdk::console;

    use super::{Erc6909, IErc6909};
    use crate::utils::introspection::erc165::IErc165;

    const TOKEN_ID: U256 = uint!(1_U256);

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc6909 as IErc6909>::interface_id();
        let expected: FixedBytes<4> = fixed_bytes!("0x0f632fb3");
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<Erc6909>, alice: Address) {
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc6909 as IErc6909>::interface_id()));
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc6909 as IErc165>::interface_id()));

        let fake_interface_id = 0x12345678u32;
        assert!(!contract
            .sender(alice)
            .supports_interface(fake_interface_id.into()));
    }

    #[motsu::test]
    fn mint(contract: Contract<Erc6909>, alice: Address) {
        contract
            .sender(alice)
            ._mint(alice, uint!(TOKEN_ID), uint!(1000_U256))
            .expect("should mint a token to Alice");

        let alice_balance =
            contract.sender(alice).balance_of(alice, uint!(TOKEN_ID));

        assert_eq!(alice_balance, uint!(1000_U256));
    }

    #[motsu::test]
    fn transfer(contract: Contract<Erc6909>, alice: Address, bob: Address) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID, uint!(1000_U256))
            .expect("should mint a token to Alice");

        contract
            .sender(alice)
            .transfer(bob, TOKEN_ID, uint!(500_U256))
            .expect("should transfer 500 tokens from Alice to Bob");

        let bob_balance = contract.sender(alice).balance_of(bob, TOKEN_ID);

        assert_eq!(bob_balance, uint!(500_U256));
    }

    #[motsu::test]
    fn transfer_from(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID, uint!(1000_U256))
            .expect("should mint a token to Alice");

        contract
            .sender(alice)
            .approve(bob, TOKEN_ID, uint!(500_U256))
            .expect("Bob should be able to spend to 300 of Alice's tokens");

        contract
            .sender(bob)
            .transfer_from(alice, charlie, TOKEN_ID, uint!(500_U256))
            .expect("should transfer 500 tokens from Alice to Bob");

        let charlie_balance =
            contract.sender(alice).balance_of(charlie, TOKEN_ID);

        assert_eq!(charlie_balance, uint!(500_U256));
    }

    #[motsu::test]
    fn burn(contract: Contract<Erc6909>, alice: Address) {
        contract
            .sender(alice)
            ._mint(alice, uint!(TOKEN_ID), uint!(1000_U256))
            .expect("should mint a token to Alice");

        contract
            .sender(alice)
            ._burn(alice, uint!(TOKEN_ID), uint!(700_U256))
            .expect("should mint a token to Alice");

        let alice_balance =
            contract.sender(alice).balance_of(alice, uint!(TOKEN_ID));

        assert_eq!(alice_balance, uint!(300_U256));
    }

    #[motsu::test]
    fn approve(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID, uint!(1000_U256))
            .expect("should mint a token to Alice");

        contract
            .sender(alice)
            .approve(bob, TOKEN_ID, uint!(300_U256))
            .expect("Bob should be able to spend to 300 of Alice's tokens");

        contract
            .sender(bob)
            .transfer_from(alice, charlie, TOKEN_ID, uint!(200_U256))
            .expect("should transfer 200 tokens from Alice to Charlie");

        let alice_balance = contract.sender(alice).balance_of(alice, TOKEN_ID);
        let charlie_balance =
            contract.sender(alice).balance_of(charlie, TOKEN_ID);

        assert_eq!(alice_balance, uint!(800_U256));
        assert_eq!(charlie_balance, uint!(200_U256));
    }

    #[motsu::test]
    fn set_operator(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID, uint!(1000_U256))
            .expect("should mint a token to Alice");

        contract
            .sender(alice)
            .set_operator(bob, true)
            .expect("Bob should become an operator of Alice's account'");

        contract
            .sender(bob)
            .transfer_from(alice, charlie, TOKEN_ID, uint!(100_U256))
            .expect("should transfer 100 tokens from Alice to Charlie");

        let alice_balance = contract.sender(alice).balance_of(alice, TOKEN_ID);
        let charlie_balance =
            contract.sender(alice).balance_of(charlie, TOKEN_ID);

        assert_eq!(alice_balance, uint!(900_U256));
        assert_eq!(charlie_balance, uint!(100_U256));
    }
}
