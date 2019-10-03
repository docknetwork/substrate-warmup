use crate::serializable_genesis::ChainSpec;
use erc20::Erc20Token;
use node_template_runtime::{
    AccountId, BabeConfig, BalancesConfig, Erc20Config, GenesisConfig, GrandpaConfig,
    IndicesConfig, SudoConfig, SystemConfig, WASM_BINARY,
};
use serde::{Deserialize, Serialize};
use substrate_consensus_babe_primitives::AuthorityId as BabeId;
use substrate_finality_grandpa_primitives::AuthorityId as GrandpaId;
use substrate_primitives::{Pair, Public};

#[derive(
    structopt::StructOpt, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize,
)]
/// generate a substrate chainspec
pub enum Chain {
    /// Outputs the chainspec for a shared testnet with a custom validator, root, and treasury
    Custom {
        #[structopt(parse(try_from_str = parse_pubkey))]
        validator_grandpa: GrandpaId,
        #[structopt(parse(try_from_str = parse_pubkey))]
        validator_babe: BabeId,
        #[structopt(parse(try_from_str = parse_pubkey))]
        root_key: AccountId,
        #[structopt(parse(try_from_str = parse_pubkey))]
        treasury: AccountId,
    },
    /// Outputs the chainspec for a testnet with Alice as validator, root, and treasury
    Ved,
}

impl Chain {
    /// Get an actual chain config from one of the alternatives.
    pub fn generate(self) -> ChainSpec<GenesisConfig> {
        match self {
            Chain::Custom {
                validator_grandpa,
                validator_babe,
                root_key,
                treasury,
            } => ChainSpec::from_genesis(
                "Substrate Warmup Custom Testnet",
                "substrate-warmup-custom",
                testnet_genesis(
                    (validator_grandpa.clone(), validator_babe.clone()),
                    root_key.clone(),
                    treasury.clone(),
                ),
                vec![],
                None,
                Some(&format!(
                    "substrate-warmup-custom-{}-{}-{}-{}",
                    validator_grandpa, validator_babe, root_key, treasury
                )),
                None,
                None,
            ),
            Chain::Ved => ChainSpec::from_genesis(
                "Substrate Warmup Local Dev Testnet",
                "substrate-warmup-local",
                testnet_genesis(
                    (
                        get_from_seed::<GrandpaId>("Alice"),
                        get_from_seed::<BabeId>("Alice"),
                    ),
                    get_from_seed::<AccountId>("Alice"),
                    get_from_seed::<AccountId>("Alice"),
                ),
                vec![],
                None,
                None,
                None,
                None,
            ),
        }
    }
}

fn testnet_genesis(
    initial_authority: (GrandpaId, BabeId),
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
            authorities: vec![(initial_authority.1, 1)],
        }),
        grandpa: Some(GrandpaConfig {
            authorities: vec![(initial_authority.0, 1)],
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
        .expect("invalid seed")
        .public()
}

fn parse_pubkey<T: Public>(imp: &str) -> Result<T, &'static str> {
    let imp: &[u8] = imp.as_bytes();

    // check key is 0x prefixed, remove prefix
    let imp: &[u8] = if imp.starts_with(b"0x") {
        &imp[2..]
    } else {
        return Err("public key shoud be prefixed with '0x'");
    };

    // check key is correct len
    if imp.len() != 64 {
        return Err("256 bit public key should be 64 hex digits");
    }

    // decode hex
    let pk: Vec<u8> = hex::decode(imp).map_err(|err| {
        use hex::FromHexError::*;
        match err {
            InvalidHexCharacter { .. } => "invalid hex character, must be [0-9][a-z][A-Z]",
            OddLength => panic!("this should not happen"),
            InvalidStringLength => panic!("this should not happen"),
        }
    })?;

    assert_eq!(pk.len(), 32);

    Ok(Public::from_slice(&pk))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_parse_pk() {
        let valid_pk = "0x6e4e511be3eae0696f542e7c05f99e5f5e7b19ce311fc8ef7c2139e0505c305c";
        parse_pubkey::<GrandpaId>(valid_pk).unwrap();
        parse_pubkey::<BabeId>(valid_pk).unwrap();
        parse_pubkey::<AccountId>(valid_pk).unwrap();
        for invalid_pk in &[
            "0x6e4e511be3eae0696f542e7c05f99e5f5e7b19ce311fc8ef7c2139e0505c305",
            "6e4e511be3eae0696f542e7c05f99e5f5e7b19ce311fc8ef7c2139e0505c305c",
            " 0x6e4e511be3eae0696f542e7c05f99e5f5e7b19ce311fc8ef7c2139e0505c305c",
            "5EZLPYKPLdfHutUAxx7hYVqwxmtjcw6MrtNygajayUDQzoSM",
            "//Alice",
            "/Alice",
            "Alice",
            "wet comic voice screen voyage hobby target prevent cluster moral menu mammal",
        ] {
            parse_pubkey::<GrandpaId>(invalid_pk).unwrap_err();
            parse_pubkey::<BabeId>(invalid_pk).unwrap_err();
            parse_pubkey::<AccountId>(invalid_pk).unwrap_err();
        }
    }

    #[test]
    // this test takes several seconds, may be worth optimizing or removing
    fn t_generate() {
        let valid_pk = "0x6e4e511be3eae0696f542e7c05f99e5f5e7b19ce311fc8ef7c2139e0505c305c";

        for chain in &[
            Chain::Custom {
                validator_grandpa: parse_pubkey::<GrandpaId>(valid_pk).unwrap(),
                validator_babe: parse_pubkey::<BabeId>(valid_pk).unwrap(),
                root_key: parse_pubkey::<AccountId>(valid_pk).unwrap(),
                treasury: parse_pubkey::<AccountId>(valid_pk).unwrap(),
            },
            Chain::Ved,
        ] {
            chain.clone().generate().into_json(true).unwrap();
            chain.clone().generate().into_json(false).unwrap();
        }
    }

    #[test]
    fn t_generate_protocol_id() {
        let valid_pk = "0x6e4e511be3eae0696f542e7c05f99e5f5e7b19ce311fc8ef7c2139e0505c305c";

        let genesis = Chain::Custom {
            validator_grandpa: parse_pubkey::<GrandpaId>(valid_pk).unwrap(),
            validator_babe: parse_pubkey::<BabeId>(valid_pk).unwrap(),
            root_key: parse_pubkey::<AccountId>(valid_pk).unwrap(),
            treasury: parse_pubkey::<AccountId>(valid_pk).unwrap(),
        }
        .generate();
        let prot_id = genesis.protocol_id().unwrap();
        assert_eq!(
            prot_id,
            "substrate-warmup-custom-\
             5EZLPYKPLdfHutUAxx7hYVqwxmtjcw6MrtNygajayUDQzoSM-\
             5EZLPYKPLdfHutUAxx7hYVqwxmtjcw6MrtNygajayUDQzoSM-\
             5EZLPYKPLdfHutUAxx7hYVqwxmtjcw6MrtNygajayUDQzoSM-\
             5EZLPYKPLdfHutUAxx7hYVqwxmtjcw6MrtNygajayUDQzoSM"
        );
    }
}
