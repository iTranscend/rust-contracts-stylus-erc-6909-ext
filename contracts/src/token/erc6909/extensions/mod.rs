//! Common extensions
pub mod content_uri;
pub mod metadata;
pub mod supply;

pub use content_uri::{Erc6909ContentUri, IErc6909ContentUri};
pub use metadata::{Erc6909Metadata, IErc6909Metadata};
pub use supply::{Erc6909Supply, IErc6909Supply};
