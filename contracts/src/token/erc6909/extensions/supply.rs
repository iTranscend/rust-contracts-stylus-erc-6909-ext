use stylus_sdk::prelude::*;

use crate::token::erc6909::Erc6909;

pub struct Erc6909Supply {
    pub erc6909: Erc6909,
}

#[interface_id]
pub trait IErc6909Supply: IErc165 {}

#[cfg(test)]
mod tests {}
