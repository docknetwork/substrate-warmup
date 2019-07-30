//! Defines Substrate chain specifications used in the project.
//! What's a ChainSpec? It's not totally clear, but substrate docs define it thusly.
//! "A configuration of a chain. Can be used to build a genesis block."

use core::iter::once;
use primitives::crypto::{DeriveJunction, DEV_PHRASE};
use primitives::{ed25519, sr25519, Pair};
use substrate_service::ChainSpec;
use substrate_warmup_runtime::{
    AccountId, AuraConfig, AuraId, BalancesConfig, GenesisConfig, IndicesConfig, SudoConfig,
    SystemConfig, WASM_BINARY,
};

/// Generate as chain spec representing the dev chain.
pub fn dev() -> ChainSpec<GenesisConfig> {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        || {
            testnet_genesis(
                vec![authority_key("Alice")],
                vec![account_key("Alice")],
                account_key("Alice"),
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

/// Generate as chain spec representing a local testnet.
pub fn local() -> ChainSpec<GenesisConfig> {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        || {
            testnet_genesis(
                vec![authority_key("Alice"), authority_key("Bob")],
                vec![
                    account_key("Alice"),
                    account_key("Bob"),
                    account_key("Charlie"),
                    account_key("Dave"),
                    account_key("Eve"),
                    account_key("Ferdie"),
                ],
                account_key("Alice"),
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

fn testnet_genesis(
    initial_authorities: Vec<AuraId>,
    endowed_accounts: Vec<AccountId>,
    root_key: AccountId,
) -> GenesisConfig {
    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        aura: Some(AuraConfig {
            authorities: initial_authorities.clone(),
        }),
        indices: Some(IndicesConfig {
            ids: endowed_accounts.clone(),
        }),
        balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
            vesting: vec![],
        }),
        sudo: Some(SudoConfig { key: root_key }),
    }
}

/// Derive Aura key using SchnorrRistrettoHDKD on a static secret
/// (substrate_primitives::crypto::DEV_PHRASE) and a single HDKD junction derived
/// from `s`.
fn authority_key(s: &str) -> AuraId {
    ed25519::Pair::from_standard_components(DEV_PHRASE, None, once(DeriveJunction::soft(s)))
        .unwrap()
        .public()
}

/// Same as authority_key, but for an AccountID
fn account_key(s: &str) -> AccountId {
    sr25519::Pair::from_standard_components(DEV_PHRASE, None, once(DeriveJunction::soft(s)))
        .unwrap()
        .public()
}
