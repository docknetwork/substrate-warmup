use crate::storage_query::{StorageQuery, ValueQuery};
use core::fmt::Debug;
use parity_scale_codec::DecodeAll;
use serde::Serialize;
use serde_json::Value;
use sr_primitives::AccountId32;
use structopt::StructOpt;
use substrate_primitives_storage::{StorageData, StorageKey};
use substrate_warmup_common::parse_accountid32;

#[derive(StructOpt, Debug)]
pub enum Key {
    Erc20(Erc20Key),
}

impl Key {
    pub fn as_raw_key(&self) -> StorageKey {
        match self {
            Key::Erc20(ek) => ek.as_raw_key(),
        }
    }

    pub fn result_scale_to_json(
        &self,
        encoded: StorageData,
    ) -> Result<Value, parity_scale_codec::Error> {
        match self {
            Key::Erc20(ek) => ek.result_scale_to_json(encoded),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum Erc20Key {
    /// Get the id of the next token to be minted. Any non-negative integer less than this value is
    /// a registered token.
    TokenId(ValueQuery<erc20::TokenId, u32>),
    // => Erc20Token<T::TokenBalance>
    Tokens {
        id: u32,
    },
    // => T::TokenBalance
    BalanceOf {
        id: u32,
        #[structopt(parse(try_from_str = parse_accountid32))]
        account: AccountId32,
    },
    // => T::TokenBalance
    Allowance {
        id: u32,
        #[structopt(parse(try_from_str = parse_accountid32))]
        sender: AccountId32,
        #[structopt(parse(try_from_str = parse_accountid32))]
        recipient: AccountId32,
    },
}

impl Erc20Key {
    pub fn as_raw_key(&self) -> StorageKey {
        match self {
            Self::TokenId(vq) => vq.to_raw_key(),
            Self::Tokens { id } => unimplemented!(),
            Self::BalanceOf { id, account } => unimplemented!(),
            Self::Allowance {
                id,
                sender,
                recipient,
            } => unimplemented!(),
        }
    }

    pub fn result_scale_to_json(
        &self,
        encoded: StorageData,
    ) -> Result<Value, parity_scale_codec::Error> {
        match self {
            Self::TokenId(vq) => vq.raw_scale_to_json(encoded),
            Self::Tokens { .. } => unimplemented!(),
            Self::BalanceOf { .. } => unimplemented!(),
            Self::Allowance { .. } => unimplemented!(),
        }
    }
}

impl StorageQuery for Erc20Key {
    type Return = (); // This doesn't really make sense

    fn to_raw_key(&self) -> StorageKey {
        match self {
            Self::TokenId(vq) => vq.to_raw_key(),
            Self::Tokens { id } => unimplemented!(),
            Self::BalanceOf { id, account } => unimplemented!(),
            Self::Allowance {
                id,
                sender,
                recipient,
            } => unimplemented!(),
        }
    }

    fn raw_scale_to_json(&self, encoded: StorageData) -> Result<Value, parity_scale_codec::Error> {
        match self {
            Self::TokenId(vq) => vq.raw_scale_to_json(encoded),
            Self::Tokens { .. } => unimplemented!(),
            Self::BalanceOf { .. } => unimplemented!(),
            Self::Allowance { .. } => unimplemented!(),
        }
    }
}
