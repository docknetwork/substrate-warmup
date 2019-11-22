use crate::storage_query::{AugmentClap, MapQuery, StorageQuery, ValueQuery};
use core::fmt::Debug;
use core::ops::Deref;
use node_template_runtime::Runtime;
use serde_json::Value;
use structopt::StructOpt;
use substrate_primitives_storage::{StorageData, StorageKey};
// use substrate_warmup_common::parse_accountid32;
// use sr_primitives::AccountId32;
// use parity_scale_codec::DecodeAll;
// use serde::Serialize;

#[derive(StructOpt, Debug)]
pub enum Key {
    /// Get the id of the next token to be minted. Any non-negative integer less than this value is
    /// a registered token.
    Erc20TokenId(ValueQuery<erc20::TokenId, u32>),
    Erc20Token(MapQuery<erc20::Tokens<Runtime>, u32, erc20::Erc20Token<u128>>),
    // Tokens {
    //     id: u32,
    // },
    // // => T::TokenBalance
    // BalanceOf {
    //     id: u32,
    //     #[structopt(parse(try_from_str = parse_accountid32))]
    //     account: AccountId32,
    // },
    // // => T::TokenBalance
    // Allowance {
    //     id: u32,
    //     #[structopt(parse(try_from_str = parse_accountid32))]
    //     sender: AccountId32,
    //     #[structopt(parse(try_from_str = parse_accountid32))]
    //     recipient: AccountId32,
    // },
}

impl StorageQuery for Key {
    fn to_raw_key(&self) -> StorageKey {
        self.deref().to_raw_key()
    }

    fn raw_scale_to_json(&self, encoded: StorageData) -> Result<Value, parity_scale_codec::Error> {
        self.deref().raw_scale_to_json(encoded)
    }
}

impl Deref for Key {
    type Target = dyn StorageQuery;

    fn deref(&self) -> &(dyn StorageQuery + 'static) {
        match self {
            Self::Erc20TokenId(q) => q,
            Self::Erc20Token(q) => q,
        }
    }
}
