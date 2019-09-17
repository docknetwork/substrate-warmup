#![cfg_attr(not(feature = "std"), no_std)]

mod erc20;

#[cfg(feature = "std")]
pub use erc20::GenesisConfig;

pub use erc20::{Event, Module, Trait, __InherentHiddenInstance};
