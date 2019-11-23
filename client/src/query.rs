use crate::json::Json;
use crate::storage_query::{AugmentClap, MapQuery, StorageQuery, ValueQuery};
use core::fmt::Debug;
use core::ops::Deref;
use node_template_runtime::Runtime;
use sr_primitives::AccountId32;
use structopt::StructOpt;
use substrate_primitives_storage::{StorageData, StorageKey};

#[derive(StructOpt, Debug)]
pub enum Key {
    /// Numerical id of the next token to be minted. Any non-negative integer less than this value
    /// is a registered token.
    Erc20TokenId(ValueQuery<erc20::TokenId, u32>),
    /// Information about a token that has alerady been minted.
	/// Takes a numerical token id as an argument.
    Erc20Token(MapQuery<erc20::Tokens<Runtime>, u32, erc20::Erc20Token<u128>>),
    /// Balance of token for account.
    /// args: `[<token-number>, "ss58address"]`
    /// example, get Alice's balance for token 0: `[0, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"]`
    Erc20BalanceOf(MapQuery<erc20::BalanceOf<Runtime>, (u32, AccountId32), u128>),
    /// Tokens granted as an allowance.
    /// args: `[<token-number>, "benefactor_account", "recipient_account"]`
    /// example, the number of unclaimed type-0 tokens Alice has granted Bob:
    /// [0, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"]
    Erc20Allowance(MapQuery<erc20::Allowance<Runtime>, (u32, AccountId32, AccountId32), u128>),
}

impl StorageQuery for Key {
    fn to_raw_key(&self) -> StorageKey {
        self.deref().to_raw_key()
    }

    fn raw_scale_to_json(&self, encoded: StorageData) -> Result<Json, parity_scale_codec::Error> {
        self.deref().raw_scale_to_json(encoded)
    }
}

impl Deref for Key {
    type Target = dyn StorageQuery;

    fn deref(&self) -> &(dyn StorageQuery + 'static) {
        match self {
            Self::Erc20TokenId(q) => q,
            Self::Erc20Token(q) => q,
            Self::Erc20BalanceOf(q) => q,
            Self::Erc20Allowance(q) => q,
        }
    }
}
