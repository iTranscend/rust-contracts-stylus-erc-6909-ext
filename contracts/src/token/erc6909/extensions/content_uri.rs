//! Extension of ERC-6909 that adds content uri request support.

use alloc::{string::String, vec, vec::Vec};

use alloy_primitives::U256;
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    prelude::*,
    storage::{StorageMap, StorageString},
};

use crate::token::erc6909::Erc6909;

/// State of an [`Erc6909ContentUri`] contract.
#[storage]
pub struct Erc6909ContentUri {
    /// [`Erc6909`] contract.
    pub erc6909: Erc6909,
    /// URI of the contract.
    pub(crate) _uri: StorageString,
    /// Mapping from token id to token uri.
    pub(crate) _token_uris: StorageMap<U256, StorageString>,
}

/// Interface for the optional ContentUri functions from the ERC-6909 standard.
#[interface_id]
pub trait IErc6909ContentUri {
    /// Returns the URI for the contract.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn contract_uri(&self) -> String;

    /// Returns the uri of a token of type `id`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id.
    fn token_uri(&self, id: U256) -> String;
}

#[public]
impl IErc6909ContentUri for Erc6909ContentUri {
    fn contract_uri(&self) -> String {
        todo!()
    }

    fn token_uri(&self, _id: U256) -> String {
        todo!()
    }
}
