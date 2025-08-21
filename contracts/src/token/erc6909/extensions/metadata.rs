//! Extension of ERC-6909 that adds metadata request support.

use alloc::{string::String, vec, vec::Vec};

use alloy_primitives::{U256, U8};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    prelude::*,
    storage::{StorageMap, StorageString, StorageU8},
};

use crate::token::erc6909::Erc6909;

/// State of an [`Erc6909Metadata`] contract.
#[storage]
pub struct Erc6909Metadata {
    /// [`Erc6909`] contract.
    pub erc6909: Erc6909,
    /// Mapping from token id to token name.
    pub(crate) _name: StorageMap<U256, StorageString>,
    /// Mapping from token id to token symbol.
    pub(crate) _symbol: StorageMap<U256, StorageString>,
    /// Mapping from token id to the amount of decimals a token has.
    pub(crate) _decimals: StorageMap<U256, StorageU8>,
}

/// Interface for the optional metadata functions from the ERC-6909 standard.
#[interface_id]
pub trait IErc6909Metadata {
    /// Returns the name for token type `id`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id.
    fn name(&self, id: U256) -> String;

    /// Returns the symbol of the token of type `id`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id.
    fn symbol(&self, id: U256) -> String;

    /// Returns the amount of decimals for token of type `id`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id.
    fn decimals(&self, id: U256) -> U8;
}

#[public]
impl IErc6909Metadata for Erc6909Metadata {
    fn name(&self, _id: U256) -> String {
        todo!()
    }

    fn symbol(&self, _id: U256) -> String {
        todo!()
    }

    fn decimals(&self, _id: U256) -> U8 {
        todo!()
    }
}
