//! StateClient provides some methods with untyped arguments or return values.
//! e.g.
//!
//! ```text
//! pub fn metadata(
//!    &self,
//!    hash: Option<Hash>
//! ) -> impl Future<Item = Bytes, Error = RpcError>;
//! ```
//!
//! This wrapper provides alternative methods with more precise types.
//!
//! ```text
//! pub fn metadata(
//!    &self,
//!    block_index: Option<Hash>
//! ) -> impl Future<Item = RuntimeMetadataPrefixed, Error = RpcError>;
//! ```

use crate::method::Method;
use futures::Future;
use jsonrpc_client_transports::RpcError;
use jsonrpc_client_transports::TypedSubscriptionStream;
use parity_scale_codec::{DecodeAll, Encode};
use serde::{de::DeserializeOwned, Serialize};
use sr_version::RuntimeVersion;
use srml_metadata::RuntimeMetadataPrefixed;
use substrate_primitives::{
    storage::{StorageChangeSet, StorageData, StorageKey},
    Bytes,
};
use substrate_rpc_api::state::StateClient;

/// Json Rpc Future
pub trait Jrf<R>: Future<Item = R, Error = RpcError> {} // Jrf<R> implies Future<Item = R, Error = RpcError>
impl<F: Future<Error = RpcError>> Jrf<F::Item> for F {} // Future<Item = R, Error = RpcError> implies Jrf<R>

pub struct WrappedClient<Client>(pub Client);

impl<Hash> WrappedClient<&StateClient<Hash>>
where
    Hash: DeserializeOwned + Serialize + Send + Sync + 'static,
{
    /// Call runtime method at given block.
    pub fn call<M: Method>(&self, args: M::Args, block_index: Option<Hash>) -> impl Jrf<M::Return> {
        // ends up calling substrate/core/rpc/src/state/state_full.rs FullState::call
        // which calls the "Runtime" directly
        self.0
            .call(M::NAME.to_string(), Bytes(args.encode()), block_index)
            .and_then(decode_scale)
    }

    /// Returns the keys with prefix, leave empty to get all the keys
    pub fn storage_keys(
        &self,
        prefix: StorageKey,
        block_index: Option<Hash>,
    ) -> impl Jrf<Vec<StorageKey>> {
        self.0.storage_keys(prefix, block_index)
    }

    /// Returns a storage entry at a specific block's state.
    pub fn storage(
        &self,
        key: StorageKey, // TODO: needs type
        block_index: Option<Hash>,
    ) -> impl Jrf<Option<StorageData>> // TODO: needs type
    {
        self.0.storage(key, block_index)
    }

    /// Returns the hash of a storage entry at a block's state.
    pub fn storage_hash(
        &self,
        key: StorageKey,
        block_index: Option<Hash>,
    ) -> impl Jrf<Option<Hash>> {
        self.0.storage_hash(key, block_index)
    }

    /// Returns the size of a storage entry at a block's state.
    pub fn storage_size(
        &self,
        key: StorageKey,
        block_index: Option<Hash>,
    ) -> impl Jrf<Option<u64>> {
        self.0.storage_size(key, block_index)
    }

    /// Returns the keys with prefix from a child storage, leave empty to get all the keys
    pub fn child_storage_keys(
        &self,
        child_storage_key: StorageKey,
        prefix: StorageKey,
        block_index: Option<Hash>,
    ) -> impl Jrf<Vec<StorageKey>> {
        self.0
            .child_storage_keys(child_storage_key, prefix, block_index)
    }

    /// Returns a child storage entry at a specific block's state.
    pub fn child_storage(
        &self,
        child_storage_key: StorageKey,
        key: StorageKey,
        block_index: Option<Hash>,
    ) -> impl Jrf<Option<StorageData>> {
        self.0.child_storage(child_storage_key, key, block_index)
    }

    /// Returns the hash of a child storage entry at a block's state.
    pub fn child_storage_hash(
        &self,
        child_storage_key: StorageKey,
        key: StorageKey,
        block_index: Option<Hash>,
    ) -> impl Jrf<Option<Hash>> {
        self.0
            .child_storage_hash(child_storage_key, key, block_index)
    }

    /// Returns the size of a child storage entry at a block's state.
    pub fn child_storage_size(
        &self,
        child_storage_key: StorageKey,
        key: StorageKey,
        block_index: Option<Hash>,
    ) -> impl Jrf<Option<u64>> {
        self.0
            .child_storage_size(child_storage_key, key, block_index)
    }

    /// Returns the runtime metadata as an opaque blob.
    pub fn metadata(&self, block_index: Option<Hash>) -> impl Jrf<RuntimeMetadataPrefixed> {
        self.0.metadata(block_index).and_then(decode_scale)
    }

    /// Get the runtime version.
    pub fn runtime_version(&self, block_index: Option<Hash>) -> impl Jrf<RuntimeVersion> {
        self.0.runtime_version(block_index)
    }

    /// Query historical storage entries (by key) starting from a block given as the second parameter.
    ///
    /// NOTE This first returned result contains the initial state of storage for all keys.
    /// Subsequent values in the vector represent changes to the previous state (diffs).
    pub fn query_storage(
        &self,
        keys: Vec<StorageKey>,
        block: Hash,
        block_index: Option<Hash>,
    ) -> impl Jrf<Vec<StorageChangeSet<Hash>>> {
        self.0.query_storage(keys, block, block_index)
    }

    /// New runtime version subscription
    pub fn subscribe_runtime_version(&self) -> impl Jrf<TypedSubscriptionStream<RuntimeVersion>> {
        self.0.subscribe_runtime_version()
    }

    /// New storage subscription
    pub fn subscribe_storage(
        &self,
        keys: Option<Vec<StorageKey>>,
    ) -> impl Jrf<TypedSubscriptionStream<StorageChangeSet<Hash>>> {
        self.0.subscribe_storage(keys)
    }
}

fn decode_scale<T: DecodeAll>(bytes: Bytes) -> Result<T, RpcError> {
    DecodeAll::decode_all(&bytes).map_err(|codec_err| {
        let bytes: &[u8] = bytes.as_ref();
        RpcError::ParseError(
            format!(
                "failure decoding scale {}",
                summarize_hex(&hex::encode(bytes))
            ),
            codec_err.into(),
        )
    })
}

/// if hex string is too long, replace the middle with elipses for display purposes
/// may panic on non-ascii characters
fn summarize_hex(inp: &str) -> String {
    if inp.len() > 16 {
        format!("{}...{}", &inp[..6], &inp[inp.len() - 6..])
    } else {
        inp.to_string()
    }
}
