use crate::serde_as_scale::SerdeAsScale;
use node_template_runtime::Runtime;

type Hash = <Runtime as srml_system::Trait>::Hash;
type BlockNumber = <Runtime as srml_system::Trait>::BlockNumber;

pub trait Method {
    /// unique static name passed as the jsonrpc "method" field of the jsonrpc call
    const NAME: &'static str;

    /// serialized to the "params" field of the jsonrpc call
    type Args: serde::Serialize;

    /// serialized to the "result" field of the jsonrpc response
    type Return: serde::de::DeserializeOwned;
}

/// this macro should be removed once const generics are stable
/// https://github.com/rust-lang/rust/issues/44580
macro_rules! declare_method {
    ($struct_name:ident, $method_name:expr, $args:ty, $return:ty) => {
        pub struct $struct_name;

        impl Method for $struct_name {
            const NAME: &'static str = $method_name;
            type Args = $args;
            type Return = $return;
        }
    };
}

/// serialzes to json as an empty list
///
/// ```
/// use client::method::NoArgs;
/// use serde_json::{to_value, json};
///
/// let a: NoArgs = [];
/// assert_eq!(to_value(&a).unwrap(), json!([]));
/// ```
pub type NoArgs = [bool; 0];

/// serializes to json as a list with a single item
///
/// ```
/// use client::method::OneArg;
/// use serde_json::{to_value, json};
///
/// let a: OneArg<bool> = [false];
/// assert_eq!(to_value(&a).unwrap(), json!([false]));
/// #
/// # assert_eq!(to_value(&[Some(false)]).unwrap(), json!([false]));
/// # let a: OneArg<Option<bool>> = [None];
/// # assert_eq!(to_value(&a).unwrap(), json!([null]));
/// ```
pub type OneArg<T> = [T; 1];

declare_method!(
    StateGetMetadata,
    "state_getMetadata",
    NoArgs,
    SerdeAsScale<srml_metadata::RuntimeMetadataPrefixed>
);
declare_method!(SystemHealth, "system_health", NoArgs, serde_json::Value);

pub mod state {
    use super::*;
    use substrate_primitives::storage::{StorageChangeSet, StorageData, StorageKey};

    declare_method!(
        GetKeys,
        "state_getKeys",
        (/* prefix */ StorageKey, /* hash */ Option<Hash>),
        Vec<StorageKey>
    );

    declare_method!(
        GetChildKeys,
        "state_getChildKeys",
        (
            /* child_storage_key */ StorageKey,
            /* prefix */ StorageKey,
            /* hash */ Option<Hash>
        ),
        Vec<StorageKey>
    );

    declare_method!(
        GetChildStorage,
        "state_getChildStorage",
        (
            /* child_storage_key */ StorageKey,
            /* key */ StorageKey,
            /* hash */ Option<Hash>
        ),
        Option<StorageData>
    );

    declare_method!(
        GetChildStorageHash,
        "state_getChildStorageHash",
        (
            /* child_storage_key */ StorageKey,
            /* key */ StorageKey,
            /* hash */ Option<Hash>
        ),
        Option<Hash>
    );

    declare_method!(
        GetChildStorageSize,
        "state_getChildStorageSize",
        (
            /* child_storage_key */ StorageKey,
            /* key */ StorageKey,
            /* hash */ Option<Hash>
        ),
        Option<u64>
    );

    declare_method!(
        GetMetadata,
        "state_getMetadata",
        OneArg<Option<Hash>>,
        substrate_primitives::Bytes
    );

    declare_method!(
        QueryStorage,
        "state_queryStorage",
        (
            /* keys */ Vec<StorageKey>,
            /* from */ Hash,
            /* to */ Option<Hash>
        ),
        Vec<StorageChangeSet<Hash>>
    );

    #[cfg(test)]
    #[test]
    fn methods() {
        let s = crate::testnode::RunningFullNode::new();

        s.remote_call::<GetKeys>((StorageKey(vec![]), Some(Hash::zero())))
            .unwrap();

        s.remote_call::<GetChildKeys>((
            /* child_storage_key */ StorageKey(vec![]),
            /* prefix */ StorageKey(vec![]),
            /* hash */ Some(Hash::zero()),
        ))
        .unwrap();

        s.remote_call::<GetChildStorage>((
            /* child_storage_key */ StorageKey(vec![]),
            /* key */ StorageKey(vec![]),
            /* hash */ Some(Hash::zero()),
        ))
        .unwrap();

        s.remote_call::<GetChildStorageHash>((
            /* child_storage_key */ StorageKey(vec![]),
            /* key */ StorageKey(vec![]),
            /* hash */ Some(Hash::zero()),
        ))
        .unwrap();

        s.remote_call::<GetChildStorageSize>((
            /* child_storage_key */ StorageKey(vec![]),
            /* key */ StorageKey(vec![]),
            /* hash */ Some(Hash::zero()),
        ))
        .unwrap();

        s.remote_call::<GetMetadata>([None]).unwrap();

        let block0 = substrate_rpc_api::chain::number::NumberOrHex::Number(0);
        let block0_hash = s
            .remote_call::<chain::GetHead>([Some(block0)])
            .unwrap()
            .unwrap();
        s.remote_call::<QueryStorage>((
            /* keys */ vec![],
            /* from */ block0_hash,
            /* to */ None,
        ))
        .unwrap();
    }
}

pub mod chain {
    use super::*;
    use chain::number::NumberOrHex;
    use sr_primitives::traits::GetRuntimeBlockType;
    use substrate_rpc_api::chain;

    type RuntimeBlock = <Runtime as GetRuntimeBlockType>::RuntimeBlock;
    type Header = <RuntimeBlock as sr_primitives::traits::Block>::Header;

    declare_method!(
        GetHeader,
        "chain_getHeader",
        OneArg</* hash */ Option<Hash>>,
        Option<Header>
    );

    declare_method!(
        GetBlock,
        "chain_getBlock",
        OneArg</* hash */ Option<Hash>>,
        Option<serde_json::Value> // Option<sr_primitives::generic::SignedBlock<RuntimeBlock>>
    );

    declare_method!(
        GetHead,
        "chain_getHead",
        OneArg</* hash */ Option<NumberOrHex<BlockNumber>>>,
        Option<Hash>
    );

    declare_method!(
        GetRuntimeVersion,
        "chain_getRuntimeVersion",
        OneArg</* hash */ Option<Hash>>,
        Option<sr_version::RuntimeVersion>
    );

    #[cfg(test)]
    #[test]
    fn methods() {
        let s = crate::testnode::RunningFullNode::new();

        s.remote_call::<GetHeader>([None]).unwrap().unwrap();

        s.remote_call::<GetBlock>([None]).unwrap().unwrap();

        s.remote_call::<GetHead>([None]).unwrap().unwrap();
        s.remote_call::<GetHead>([Some(NumberOrHex::Number(0))])
            .unwrap()
            .unwrap();

        s.remote_call::<GetRuntimeVersion>([None]).unwrap().unwrap();
    }
}

pub mod author {
    use super::*;
    use substrate_rpc_api::author::hash::ExtrinsicOrHash;

    declare_method!(
        SubmitExtrinsic,
        "author_submitExtrinsic",
        OneArg</* extrinsic */ substrate_primitives::Bytes>,
        Hash
    );

    declare_method!(
        InsertKey,
        "author_insertKey",
        (
            /* key_type */ String,
            /* suri */ String,
            /* public */ substrate_primitives::Bytes
        ),
        ()
    );

    declare_method!(
        RotateKeys,
        "author_rotateKeys",
        NoArgs,
        substrate_primitives::Bytes
    );

    declare_method!(
        PendingExtrinsics,
        "author_pendingExtrinsics",
        NoArgs,
        Vec<substrate_primitives::Bytes>
    );

    declare_method!(
        RemoveExtrinsic,
        "author_removeExtrinsic",
        OneArg</* bytes_or_hash */ Vec<ExtrinsicOrHash<Hash>>>,
        Vec<Hash>
    );

    #[cfg(test)]
    #[test]
    fn methods() {
        let s = crate::testnode::RunningFullNode::new();

        s.remote_call::<SubmitExtrinsic>([substrate_primitives::Bytes(vec![
            // invalid transaction, erc20 transfer 132 of token 0 from alice to bddap
            0x71, 0x02, 0x83, 0xff, 0xd4, 0x35, 0x93, 0xc7, 0x15, 0xfd, 0xd3, 0x1c, 0x61, 0x14,
            0x1a, 0xbd, 0x04, 0xa9, 0x9f, 0xd6, 0x82, 0x2c, 0x85, 0x58, 0x85, 0x4c, 0xcd, 0xe3,
            0x9a, 0x56, 0x84, 0xe7, 0xa5, 0x6d, 0xa2, 0x7d, 0x38, 0x73, 0x7a, 0xc6, 0x67, 0x71,
            0xe7, 0x7b, 0x6f, 0x6c, 0xdb, 0x15, 0x2e, 0x1d, 0xbf, 0x77, 0x0b, 0xf2, 0xa7, 0x52,
            0x6f, 0x37, 0x03, 0x89, 0x43, 0xaa, 0xe4, 0x73, 0xff, 0xd4, 0x58, 0x49, 0xf4, 0x79,
            0xdb, 0x2e, 0x53, 0xf4, 0xd4, 0x6d, 0xa5, 0xa6, 0xe8, 0x7a, 0x52, 0x2d, 0x40, 0xc1,
            0x53, 0x41, 0x6a, 0xb3, 0xf1, 0x46, 0x1b, 0x91, 0xdf, 0x90, 0x37, 0xcc, 0x01, 0xff,
            0xeb, 0x04, 0xe5, 0x03, 0x1c, 0x00, 0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0xa0, 0x3b,
            0x8c, 0xc0, 0xdd, 0xef, 0x78, 0x7d, 0x1c, 0x5a, 0x84, 0x71, 0x27, 0x58, 0x9b, 0xb2,
            0xb1, 0x64, 0x08, 0x8d, 0x5c, 0x67, 0xf1, 0xda, 0x49, 0xb2, 0xdb, 0x0b, 0x04, 0x1d,
            0x78, 0x4d, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ])])
        .unwrap_err();

        s.remote_call::<InsertKey>((
            /* key_type */ "four".to_string(),
            /* suri */
            "omit file figure place ecology member quiz series poet group desert snack".to_string(),
            /* public */
            substrate_primitives::Bytes(vec![
                0x60, 0x31, 0x80, 0x12, 0x50, 0xaf, 0xa8, 0xd5, 0x92, 0xc2, 0x89, 0x7b, 0xcf, 0x91,
                0x77, 0x88, 0x45, 0xb2, 0x84, 0x3d, 0x89, 0xe6, 0xba, 0x30, 0x8c, 0x01, 0x0b, 0x18,
                0x8c, 0xec, 0xb1, 0x01,
            ]),
        ))
        .unwrap();

        s.remote_call::<RotateKeys>([]).unwrap();

        s.remote_call::<PendingExtrinsics>([]).unwrap();

        s.remote_call::<RemoveExtrinsic>([vec![]]).unwrap();
    }
}

pub mod system {
    use super::*;
    use substrate_rpc_api::system;

    declare_method!(Name, "system_name", NoArgs, String);

    declare_method!(Version, "system_version", NoArgs, String);

    declare_method!(Chain, "system_chain", NoArgs, String);

    declare_method!(Properties, "system_properties", NoArgs, system::Properties);

    declare_method!(Health, "system_health", NoArgs, system::Health);

    declare_method!(
        Peers,
        "system_peers",
        NoArgs,
        Vec<system::PeerInfo<Hash, BlockNumber>>
    );

    declare_method!(
        NetworkState,
        "system_networkState",
        NoArgs,
        serde_json::Value
    );

    #[cfg(test)]
    #[test]
    fn methods() {
        let s = crate::testnode::RunningFullNode::new();

        s.remote_call::<Name>([]).unwrap();

        s.remote_call::<Version>([]).unwrap();

        s.remote_call::<Chain>([]).unwrap();

        s.remote_call::<Properties>([]).unwrap();

        s.remote_call::<Health>([]).unwrap();

        s.remote_call::<Peers>([]).unwrap();

        s.remote_call::<NetworkState>([]).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testnode::RunningFullNode;

    #[test]
    fn methods() {
        let server = RunningFullNode::new();
        server.remote_call::<StateGetMetadata>([]).unwrap();
        server.remote_call::<SystemHealth>([]).unwrap();
    }

    #[test]
    fn bad_extrinsic_input() {
        let s = RunningFullNode::new();

        // panicked at 'Bad input data provided to validate_transaction: ', /Users/a/d/substrate-warmup/runtime/src/runtime.rs:299:1
        let _ = s.remote_call::<author::SubmitExtrinsic>([substrate_primitives::Bytes(
            b"definitely a valid extrinsic".to_vec(),
        )]);
    }

    #[test]
    #[ignore]
    fn valid_extrinsic() {
        let s = RunningFullNode::new();

        s.remote_call::<author::SubmitExtrinsic>([substrate_primitives::Bytes(b"todo".to_vec())])
            .unwrap();
    }

    /// RotateKeys sometimes dumps files in the CWD
    #[test]
    fn rotate_key_dumps_files() {
        use std::collections::BTreeSet;
        use std::fs::read_dir;

        let path = |rd: std::io::Result<std::fs::DirEntry>| rd.unwrap().path();
        let ls = || -> BTreeSet<_> { read_dir("./").unwrap().map(path).collect() };

        let s = RunningFullNode::new();
        let files_before = ls();
        s.remote_call::<author::RotateKeys>([]).unwrap();
        let files_after = ls();
        assert_eq!(files_before, files_after);
    }
}

// #!/usr/bin/env python3
//
// import sys
// import re
//
// txt = sys.stdin.read() # cat $(rg -l .)
//
// reg = re.compile(r"#\[\ *rpc\(\ *[^)]*\)\ *\][^;\{]*")
//
// mat = [txt[ite.start():ite.end()] for ite in reg.finditer(txt)]
//
// print(*mat, sep='\n\n')

// this method was tested, "method not found" was returned
// pub mod account {
//     use super::*;
//     declare_method!(
//         NextIndex,
//         "account_nextIndex",
//         OneArg</* account */ <Runtime as srml_system::Trait>::AccountId>,
//         <Runtime as srml_system::Trait>::Index
//     );
// }
