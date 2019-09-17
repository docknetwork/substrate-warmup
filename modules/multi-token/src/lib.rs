#![cfg_attr(not(feature = "std"), no_std)]

mod multi_token;

#[cfg(feature = "std")]
pub use multi_token::GenesisConfig;

pub use multi_token::{Event, Module, Trait, __InherentHiddenInstance};
