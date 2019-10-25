// `construct_runtime!`, used in runtime.rs, is highly recursive
#![recursion_limit = "256"]
#![cfg_attr(not(feature = "std"), no_std)]

mod runtime;

// In substrate land, #[cfg(feature = "std")] is the de-facto way to determine whether we are
// compiling a wasm runtime.
//
// The following exports only exists when compiling with feature = "std".
#[cfg(feature = "std")]
pub use runtime::{
    native_version, BabeConfig, BalancesConfig, Erc20Config, GenesisConfig, GrandpaConfig,
    IndicesConfig, SudoConfig, SystemConfig, WASM_BINARY,
};

// The following is only made public only when compiling with feature = "std".
#[cfg(feature = "std")]
pub use runtime::{api, opaque, AccountId, Call, Runtime, RuntimeApi};

#[cfg(test)]
mod tests {
    use super::GenesisConfig;

    use primitives::Blake2Hasher;
    use runtime_io::with_externalities;
    use sr_primitives::BuildStorage as _;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        GenesisConfig {
            babe: None,
            balances: None,
            grandpa: None,
            indices: None,
            sudo: None,
            system: None,
            erc20: None,
        }
        .build_storage()
        .unwrap()
        .into()
    }

    #[test]
    #[ignore] // not yet implemented
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            unimplemented!()
            // // Just a dummy test for the dummy funtion `do_something`
            // // calling the `do_something` function with a value 42
            // assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
            // // asserting that the stored value is equal to what we stored
            // assert_eq!(TemplateModule::something(), Some(42));
        });
    }

    #[test]
    #[ignore] // not yet implemented
    fn call_wasm_runtime() {
        // create test-chain
        // load and use wasm module
        unimplemented!();
    }
}
