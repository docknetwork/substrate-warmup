use super::Chain;
use erc20::Erc20Token;
use node_template_runtime::{
    AccountId, BabeConfig, BalancesConfig, Erc20Config, GenesisConfig, GrandpaConfig,
    IndicesConfig, SudoConfig, SystemConfig, WASM_BINARY,
};
use substrate_consensus_babe_primitives::AuthorityId as BabeId;
use substrate_finality_grandpa_primitives::AuthorityId as GrandpaId;
use substrate_primitives::{Pair, Public};
use substrate_service;

/// Get an actual chain config from one of the alternatives.
pub(super) fn generate(chain: Chain) -> substrate_service::ChainSpec<GenesisConfig> {
    match chain {
        Chain::Ent => substrate_service::ChainSpec::from_genesis(
            "Ent Shared Dev Testnet",
            "dock-ent",
            || {
                let bddap_keys: (AccountId, AccountId, GrandpaId, BabeId) = (
                    Public::from_slice(&[
                        0xe6, 0x9f, 0x08, 0x8a, 0xa0, 0x18, 0xf6, 0xe9, 0xd7, 0x48, 0x2d, 0x83,
                        0x1f, 0x5d, 0x0b, 0x04, 0xf9, 0xe7, 0xdb, 0xf6, 0xca, 0xcf, 0x75, 0xf3,
                        0xdb, 0x5b, 0xd7, 0x5c, 0xa8, 0x58, 0xbb, 0x0a,
                    ]),
                    Public::from_slice(&[
                        0xb2, 0xae, 0x55, 0x75, 0xda, 0x1f, 0xdc, 0x94, 0x82, 0x13, 0x15, 0x9e,
                        0x76, 0xf4, 0x59, 0x4e, 0x04, 0xc8, 0xd6, 0x36, 0x02, 0x44, 0x5a, 0x00,
                        0x0c, 0x7d, 0xe4, 0x82, 0xeb, 0xe4, 0x2d, 0x76,
                    ]),
                    Public::from_slice(&[
                        0xd7, 0x24, 0x7c, 0x76, 0xce, 0x63, 0x0a, 0x91, 0xb3, 0x62, 0xe6, 0xec,
                        0x78, 0x8e, 0xe6, 0x1c, 0xc7, 0x35, 0xb1, 0xae, 0xa7, 0xca, 0x85, 0x02,
                        0xc8, 0x68, 0xba, 0xe7, 0xf6, 0x52, 0x00, 0x2b,
                    ]),
                    Public::from_slice(&[
                        0xb2, 0xae, 0x55, 0x75, 0xda, 0x1f, 0xdc, 0x94, 0x82, 0x13, 0x15, 0x9e,
                        0x76, 0xf4, 0x59, 0x4e, 0x04, 0xc8, 0xd6, 0x36, 0x02, 0x44, 0x5a, 0x00,
                        0x0c, 0x7d, 0xe4, 0x82, 0xeb, 0xe4, 0x2d, 0x76,
                    ]),
                );
                let bddap_sudo = Public::from_slice(&[
                    0xb2, 0xae, 0x55, 0x75, 0xda, 0x1f, 0xdc, 0x94, 0x82, 0x13, 0x15, 0x9e, 0x76,
                    0xf4, 0x59, 0x4e, 0x04, 0xc8, 0xd6, 0x36, 0x02, 0x44, 0x5a, 0x00, 0x0c, 0x7d,
                    0xe4, 0x82, 0xeb, 0xe4, 0x2d, 0x76,
                ]);
                let bddap_treasury = Public::from_slice(&[
                    0xb2, 0xae, 0x55, 0x75, 0xda, 0x1f, 0xdc, 0x94, 0x82, 0x13, 0x15, 0x9e, 0x76,
                    0xf4, 0x59, 0x4e, 0x04, 0xc8, 0xd6, 0x36, 0x02, 0x44, 0x5a, 0x00, 0x0c, 0x7d,
                    0xe4, 0x82, 0xeb, 0xe4, 0x2d, 0x76,
                ]);
                testnet_genesis(vec![bddap_keys], bddap_sudo, bddap_treasury)
            },
            vec![],
            None,
            Some("dock-substrate-warmup-ent"),
            None,
            None,
        ),
        Chain::Ved => substrate_service::ChainSpec::from_genesis(
            "Ved Local Dev Testnet",
            "dock-ved",
            || {
                testnet_genesis(
                    vec![get_authority_keys_from_seed("Alice")],
                    get_from_seed::<AccountId>("Alice"),
                    get_from_seed::<AccountId>("Alice"),
                )
            },
            vec![],
            None,
            None,
            None,
            None,
        ),
    }
}

fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
    root_key: AccountId,
    treasury: AccountId,
) -> GenesisConfig {
    const ENDOWMENT: u128 = u128::max_value();

    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        indices: Some(IndicesConfig {
            ids: vec![treasury.clone()],
        }),
        balances: Some(BalancesConfig {
            balances: vec![(treasury.clone(), ENDOWMENT)],
            vesting: vec![],
        }),
        sudo: Some(SudoConfig { key: root_key }),
        babe: Some(BabeConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.3.clone(), 1))
                .collect(),
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.2.clone(), 1))
                .collect(),
        }),
        erc20: Some(Erc20Config {
            initial_tokens: vec![
                (
                    Erc20Token {
                        name: b"PSTABLE1".to_vec(),
                        ticker: b"PSTABLE1".to_vec(),
                        total_supply: ENDOWMENT,
                    },
                    treasury.clone(),
                ),
                (
                    Erc20Token {
                        name: b"PSTABLE2".to_vec(),
                        ticker: b"PSTABLE2".to_vec(),
                        total_supply: ENDOWMENT,
                    },
                    treasury.clone(),
                ),
            ],
        }),
    }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<P: Public>(seed: &str) -> <P::Pair as Pair>::Public {
    P::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, GrandpaId, BabeId) {
    (
        get_from_seed::<AccountId>(&format!("{}//stash", seed)),
        get_from_seed::<AccountId>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
    )
}
