#![cfg_attr(not(feature = "std"), no_std)]

mod erc20;

#[cfg(feature = "std")]
pub use crate::erc20::GenesisConfig;

pub use crate::erc20::{Erc20Token, Event, Module, Trait, __InherentHiddenInstance};
