#![cfg_attr(not(feature = "std"), no_std)]

mod erc20;

#[cfg(feature = "std")]
pub use crate::erc20::GenesisConfig;

pub use crate::erc20::{
    Allowance, BalanceOf, Erc20Token, Event, Module, TokenId, Tokens, Trait,
    __InherentHiddenInstance,
};
