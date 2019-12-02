use crate::json::Json;
use crate::storage_query::{AugmentClap, MapQuery, StorageQuery, ValueQuery};
use core::fmt::Debug;
use core::ops::Deref;
use node_template_runtime::Runtime;
use sr_primitives::AccountId32;
use structopt::StructOpt;
use substrate_consensus_babe_primitives::BabeAuthorityWeight;
use substrate_primitives::H256;
use substrate_primitives_storage::{StorageData, StorageKey};

#[derive(StructOpt, Debug)]
/// Key arguements should be provided as json.
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

    /// Extrinsics nonce for accounts.
    /// example, the nonce expected in Alice's next signed transaction:
    /// "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    SystemAccountNonce(MapQuery<srml_system::AccountNonce<Runtime>, AccountId32, u32>),
    /// Map of block numbers to block hashes.
    SystemBlockHash(MapQuery<srml_system::BlockHash<Runtime>, u32, H256>),

    TimestampNow(ValueQuery<srml_timestamp::Now<Runtime>, u64>),

    BabeEpochIndex(ValueQuery<srml_babe::EpochIndex, u64>),
    BabeAuthorities(
        ValueQuery<srml_babe::Authorities, Vec<(srml_babe::AuthorityId, BabeAuthorityWeight)>>,
    ),
    BabeGenesisSlot(ValueQuery<srml_babe::GenesisSlot, u64>),
    BabeCurrentSlot(ValueQuery<srml_babe::CurrentSlot, u64>),
    BabeRandomness(ValueQuery<srml_babe::Randomness, [u8; 32]>),

    BalancesTotalIssuance(ValueQuery<srml_balances::TotalIssuance<Runtime>, u128>),
    // BalanceLock and VestingSchedule do not implement Serde so cannot be serialized to json.
    BalancesFreeBalance(MapQuery<srml_balances::FreeBalance<Runtime>, AccountId32, u128>),
    BalancesReservedBalance(MapQuery<srml_balances::ReservedBalance<Runtime>, AccountId32, u128>),

    /// The map of all vote records indexed by id
    VotingVoteRecords(MapQuery<voting::VoteRecords<Runtime>, u64, voting::VoteRecord<AccountId32>>),
    /// The number of vote records that have been created
    VotingVoteRecordCount(ValueQuery<voting::VoteRecordCount, u64>),
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
            Self::SystemAccountNonce(q) => q,
            Self::SystemBlockHash(q) => q,
            Self::TimestampNow(q) => q,
            Self::BabeEpochIndex(q) => q,
            Self::BabeAuthorities(q) => q,
            Self::BabeGenesisSlot(q) => q,
            Self::BabeCurrentSlot(q) => q,
            Self::BabeRandomness(q) => q,
            Self::BalancesTotalIssuance(q) => q,
            Self::BalancesFreeBalance(q) => q,
            Self::BalancesReservedBalance(q) => q,
            Self::VotingVoteRecords(q) => q,
            Self::VotingVoteRecordCount(q) => q,
        }
    }
}
