use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    prelude::*,
    storage::{StorageBool, StorageMap, StorageU256},
};

use crate::utils::introspection::erc165::IErc165;

pub mod extensions;

pub use sol::*;
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
        /// Thrown when owner balance for id is insufficient.
        ///
        /// * `owner` - Address of the owner of the token.
        /// * `id` - Token id as a number.
        #[derive(Debug)]
        error Erc6909InsufficientBalance(
            address owner,
            uint256 id
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

        /// Indicates a failure with the token `receiver`.
        /// Used in transfers.
        ///
        /// * `receiver` - Address to which tokens are being transferred.
        #[derive(Debug)]
        error ERC6909InvalidReceiver(address receiver);
    }
}

#[derive(SolidityError, Debug)]
pub enum Error {
    // Indicate an owner's token balance is insufficient
    InsufficientBalance(Erc6909InsufficientBalance),
    // Indicates the spender does not have permission to spend the token.
    InsufficientPermission(Erc6909InsufficientPermission),
    // Indicates the approver is invalid.
    InvalidApprover(ERC6909InvalidApprover),
    // Indicates the sender is invalid.
    InvalidSender(ERC6909InvalidSender),
    // Indicates the receiver is invalid.
    InvalidReceiver(ERC6909InvalidReceiver),
}

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
