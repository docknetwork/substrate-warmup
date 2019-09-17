use babe_primitives::AuthorityId as BabeId;
use grandpa_primitives::AuthorityId as GrandpaId;
use node_template_runtime::{
    AccountId, BabeConfig, BalancesConfig, Erc20Config, GenesisConfig, GrandpaConfig,
    IndicesConfig, SudoConfig, SystemConfig, TokenType, WASM_BINARY,
};
use primitives::{Pair, Public};
use substrate_service;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    Ent, // shared testnet with bddap as validator
    Ved, // testnet with Alice as validator
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

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Alternative::Ent => ChainSpec::from_genesis(
                "Ent Testnet",
                "ent",
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
                        0xb2, 0xae, 0x55, 0x75, 0xda, 0x1f, 0xdc, 0x94, 0x82, 0x13, 0x15, 0x9e,
                        0x76, 0xf4, 0x59, 0x4e, 0x04, 0xc8, 0xd6, 0x36, 0x02, 0x44, 0x5a, 0x00,
                        0x0c, 0x7d, 0xe4, 0x82, 0xeb, 0xe4, 0x2d, 0x76,
                    ]);
                    testnet_genesis(vec![bddap_keys], bddap_sudo, vec![])
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::Ved => ChainSpec::from_genesis(
                "Ved Testnet",
                "ved",
                || {
                    testnet_genesis(
                        vec![get_authority_keys_from_seed("Alice")],
                        get_from_seed::<AccountId>("Alice"),
                        vec![get_from_seed::<AccountId>("Alice")],
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "ent" => Some(Alternative::Ent),
            "ved" => Some(Alternative::Ved),
            _ => None,
        }
    }
}

fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    const ENDOWMENT: u128 = 1 << 60;

    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        indices: Some(IndicesConfig {
            ids: endowed_accounts.clone(),
        }),
        balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, ENDOWMENT))
                .collect(),
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
            balances: {
                let mut ret = Vec::new();
                for token in &[TokenType::PDock, TokenType::PStable] {
                    ret.extend(
                        endowed_accounts
                            .iter()
                            .cloned()
                            .map(|account| ((*token, account), ENDOWMENT))
                            .collect::<Vec<_>>(),
                    );
                }
                ret
            },
        }),
    }
}
