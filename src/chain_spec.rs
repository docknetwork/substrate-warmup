use erc20::Erc20Token;
use node_template_runtime::{
    AuraConfig, BalancesConfig, Erc20Config, GenesisConfig, GrandpaConfig, SudoConfig,
    SystemConfig, WASM_BINARY,
};
use serde::{Deserialize, Serialize};
use sr_primitives::AccountId32;
use structopt::StructOpt;
use substrate_chain_spec::ChainSpec;
use substrate_consensus_aura_primitives::sr25519::AuthorityId as AuraId;
use substrate_finality_grandpa_primitives::AuthorityId as GrandpaId;
use substrate_primitives::sr25519;
use substrate_primitives::{Pair, Public};

#[derive(StructOpt, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
/// generate a substrate chainspec
pub enum Chain {
    /// Outputs the chainspec for a shared testnet with a custom validator, root, and treasury
    Custom {
        #[structopt(parse(try_from_str = parse_pubkey))]
        validator_grandpa: GrandpaId,
        #[structopt(parse(try_from_str = parse_pubkey))]
        validator_aura: AuraId,
        #[structopt(parse(try_from_str = parse_accountid32))]
        root_key: AccountId32,
        #[structopt(parse(try_from_str = parse_accountid32))]
        treasury: AccountId32,
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
                validator_aura,
                root_key,
                treasury,
            } => {
                let protocol_id: String = format!(
                    "substrate-warmup-custom-{}-{}-{}-{}",
                    &validator_grandpa, &validator_aura, &root_key, &treasury
                );
                ChainSpec::from_genesis(
                    "Substrate Warmup Custom Testnet",
                    "substrate-warmup-custom",
                    move || {
                        testnet_genesis(
                            (validator_grandpa.clone(), validator_aura.clone()),
                            root_key.clone(),
                            treasury.clone(),
                        )
                    },
                    vec![],
                    None,
                    Some(&protocol_id),
                    None,
                    None,
                )
            }
            Chain::Ved => ChainSpec::from_genesis(
                "Substrate Warmup Local Dev Testnet",
                "substrate-warmup-local",
                || {
                    testnet_genesis(
                        (
                            get_from_seed::<GrandpaId>("Alice"),
                            get_from_seed::<AuraId>("Alice"),
                        ),
                        id32_from_sr_seed("Alice"),
                        id32_from_sr_seed("Alice"),
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
}

fn testnet_genesis(
    initial_authority: (GrandpaId, AuraId),
    root_key: AccountId32,
    treasury: AccountId32,
) -> GenesisConfig {
    const ENDOWMENT: u128 = u128::max_value();

    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        balances: Some(BalancesConfig {
            balances: vec![(treasury.clone(), ENDOWMENT)],
            vesting: vec![],
        }),
        sudo: Some(SudoConfig { key: root_key }),
        aura: Some(AuraConfig {
            authorities: vec![initial_authority.1],
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

/// Derive sr25519 key from private key seed, return corresponding publik key as
/// an AccountId32.
fn id32_from_sr_seed(seed: &str) -> AccountId32 {
    AccountId32::from(get_from_seed::<sr25519::Public>(seed).0)
}

/// convert a 0x prefixed hex string to a 32 byte public key
fn parse_pubkey<T: Public>(imp: &str) -> Result<T, &'static str> {
    Ok(Public::from_slice(&parse_key_256(imp)?))
}

/// convert a 0x prefixed hex string to an AccountId32
fn parse_accountid32(imp: &str) -> Result<AccountId32, &'static str> {
    parse_key_256(imp).map(Into::into)
}

/// panics if slice is wrong length
fn slice_to_arr32<T: Default + Copy>(src: &[T]) -> [T; 32] {
    let mut ret: [T; 32] = Default::default();
    ret.copy_from_slice(src);
    ret
}

/// parse a 256 bit, 32 byte key from a 0x prefixed hex string
fn parse_key_256(imp: &str) -> Result<[u8; 32], &'static str> {
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

    Ok(slice_to_arr32(&pk))
}

#[cfg(test)]
mod tests {
    use super::*;
    use node_template_runtime::Runtime;

    #[test]
    fn t_parse_pk() {
        let valid_pk = "0x6e4e511be3eae0696f542e7c05f99e5f5e7b19ce311fc8ef7c2139e0505c305c";
        parse_pubkey::<GrandpaId>(valid_pk).unwrap();
        parse_pubkey::<AuraId>(valid_pk).unwrap();
        parse_pubkey::<sr25519::Public>(valid_pk).unwrap();
        parse_accountid32(valid_pk).unwrap();

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
            parse_pubkey::<AuraId>(invalid_pk).unwrap_err();
            parse_pubkey::<sr25519::Public>(invalid_pk).unwrap_err();
            parse_accountid32(invalid_pk).unwrap_err();
        }
    }

    #[test]
    // this test takes several seconds, may be worth optimizing or removing
    fn t_generate() {
        let valid_pk = "0x6e4e511be3eae0696f542e7c05f99e5f5e7b19ce311fc8ef7c2139e0505c305c";

        for chain in &[
            Chain::Custom {
                validator_grandpa: parse_pubkey::<GrandpaId>(valid_pk).unwrap(),
                validator_aura: parse_pubkey::<AuraId>(valid_pk).unwrap(),
                root_key: parse_accountid32(valid_pk).unwrap(),
                treasury: parse_accountid32(valid_pk).unwrap(),
            },
            Chain::Ved,
        ] {
            chain.clone().generate().to_json(true).unwrap();
            chain.clone().generate().to_json(false).unwrap();
        }
    }

    #[test]
    fn t_generate_protocol_id() {
        let valid_pk = "0x6e4e511be3eae0696f542e7c05f99e5f5e7b19ce311fc8ef7c2139e0505c305c";

        let genesis = Chain::Custom {
            validator_grandpa: parse_pubkey::<GrandpaId>(valid_pk).unwrap(),
            validator_aura: parse_pubkey::<AuraId>(valid_pk).unwrap(),
            root_key: parse_accountid32(valid_pk).unwrap(),
            treasury: parse_accountid32(valid_pk).unwrap(),
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

    #[test]
    fn account_id_is_system_account_id() {
        use std::any::TypeId;
        assert_eq!(
            TypeId::of::<<Runtime as srml_system::Trait>::AccountId>(),
            TypeId::of::<AccountId32>()
        );
    }
}
