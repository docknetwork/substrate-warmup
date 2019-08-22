//! Defines Substrate chain specifications used in the project.
//! What's a ChainSpec? It's not totally clear, but substrate docs define it thusly.
//! "A configuration of a chain. Can be used to build a genesis block."

use core::iter::once;
use runtime::{AuraConfig, GenesisConfig, IndicesConfig, SystemConfig, WASM_BINARY};
use substrate_primitives::crypto::{DeriveJunction, DEV_PHRASE};
use substrate_primitives::{ed25519, Pair};
use substrate_service::ChainSpec;

const AURA_PK: ed25519::Public = ed25519::Public([
    0x03, 0xba, 0x7b, 0x33, 0x55, 0xcc, 0x1b, 0x4e, 0x48, 0x4b, 0x7d, 0xe0, 0x67, 0xda, 0x32, 0xe1,
    0x73, 0xb9, 0x03, 0xb8, 0x81, 0x33, 0x4f, 0x55, 0xcf, 0x2c, 0x3b, 0x7b, 0x63, 0xc4, 0x73, 0xd3,
]);

/// Generate as chain spec representing the dev chain.
pub fn dev() -> ChainSpec<GenesisConfig> {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        || testnet_genesis(vec![AURA_PK], Vec::new()),
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
                vec![dev_pk("Alice"), dev_pk("Bob")],
                vec![
                    dev_pk("Alice"),
                    dev_pk("Bob"),
                    dev_pk("Charlie"),
                    dev_pk("Dave"),
                    dev_pk("Eve"),
                    dev_pk("Ferdie"),
                ],
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
    initial_authorities: Vec<ed25519::Public>,
    endowed_accounts: Vec<ed25519::Public>,
) -> GenesisConfig {
    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        srml_aura: Some(AuraConfig {
            authorities: initial_authorities.clone(),
        }),
        srml_indices: Some(IndicesConfig {
            ids: endowed_accounts.clone(),
        }),
        // PDOCK initial allocations
        srml_balances_Instance0: Some(srml_balances::GenesisConfig {
            balances: Vec::new(),
            vesting: Vec::new(),
        }),
        // PSTABLE initial allocations
        srml_balances_Instance1: Some(srml_balances::GenesisConfig {
            balances: Vec::new(),
            vesting: Vec::new(),
        }),
    }
}

/// Derive ed25519 key using SchnorrRistrettoHDKD on a static secret
/// (substrate_primitives::crypto::DEV_PHRASE) and a single hard junction derived from `s`.
fn dev_pk(s: &str) -> ed25519::Public {
    ed25519::Pair::from_standard_components(DEV_PHRASE, None, once(DeriveJunction::hard(s)))
        .expect("err generating authority key")
        .public()
}

#[cfg(test)]
mod test {
    use super::dev_pk;

    #[test]
    fn derive_dev_pk() {
        for name in &["Alice", "/Alice", "//Alice", "1", "0"] {
            dbg!(name);
            dev_pk(name);
        }
    }
}
