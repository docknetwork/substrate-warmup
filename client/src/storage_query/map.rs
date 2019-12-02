use super::augment_clap::AugmentClap;
use super::Json;
use crate::storage_query::StorageQuery;
use core::fmt::Debug;
use core::marker::PhantomData;
use parity_scale_codec::FullCodec;
use serde::{de::DeserializeOwned, Serialize};
use srml_support::storage::generator::StorageMap;
use structopt::clap::{self, App, ArgMatches};
use structopt::StructOpt;
use substrate_primitives_storage::{StorageData, StorageKey};

pub struct MapQuery<S, K, V> {
    key: K,
    _spook: PhantomData<(S, V)>,
}

impl<S, K: Debug, V> Debug for MapQuery<S, K, V> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(&self.key, fmt)
    }
}

impl<S: StorageMap<K, V>, K: FullCodec, V: FullCodec + Serialize> StorageQuery
    for MapQuery<S, K, V>
{
    fn to_raw_key(&self) -> StorageKey {
        StorageKey(S::storage_map_final_key(&self.key).as_ref().to_vec())
    }

    fn raw_scale_to_json(&self, raw: StorageData) -> Result<Json, parity_scale_codec::Error> {
        super::raw_scale_to_json::<V>(raw)
    }
}

impl<S, K: DeserializeOwned, V> StructOpt for MapQuery<S, K, V> {
    fn clap<'a, 'b>() -> App<'a, 'b> {
        Self::augment_clap(App::new(""))
    }

    fn from_clap(matches: &ArgMatches<'_>) -> Self {
        Self {
            key: matches
                .value_of("key")
                .map(|s| serde_json::from_str(s).unwrap())
                .unwrap(),
            _spook: PhantomData,
        }
    }
}

impl<S, K: DeserializeOwned, V> AugmentClap for MapQuery<S, K, V> {
    fn augment_clap<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app.arg(
            clap::Arg::with_name("key")
                .takes_value(true)
                .multiple(false)
                .required(true)
                .validator(|s| {
                    serde_json::from_str(s.as_str())
                        .map(|_: K| ())
                        .map_err(|e| e.to_string())
                }),
        )
    }
}
