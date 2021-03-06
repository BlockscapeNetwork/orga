#![feature(trait_alias)]
#![feature(fn_traits)]
#![feature(proc_macro_hygiene)]
// #![feature(optin_builtin_traits)]
#![feature(auto_traits)]

#[cfg(feature = "abci")]
pub mod abci;
pub mod collections;
mod encoding;
pub mod error;
#[cfg(feature = "merk")]
pub mod merkstore;
mod state;
mod state_machine;
mod store;

pub use encoding::*;
pub use error::*;
pub use state::*;
pub use state_machine::*;
pub use store::{split, *};

pub use orga_macros::*;
