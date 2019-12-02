use super::augment_clap::AugmentClap;
use super::Json;
use crate::storage_query::StorageQuery;
use core::fmt::Debug;
use core::marker::PhantomData;
use parity_scale_codec::FullCodec;
use serde::Serialize;
use srml_support::storage::generator::StorageValue;
use structopt::clap::{App, ArgMatches};
use structopt::StructOpt;
use substrate_primitives_storage::{StorageData, StorageKey};

pub struct ValueQuery<S, V> {
    _spook: PhantomData<(S, V)>,
}

impl<S, V> Debug for ValueQuery<S, V> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(&(), fmt)
    }
}

impl<S: StorageValue<V>, V: FullCodec + Serialize> StorageQuery for ValueQuery<S, V> {
    fn to_raw_key(&self) -> StorageKey {
        StorageKey(S::storage_value_final_key().to_vec())
    }

    fn raw_scale_to_json(&self, raw: StorageData) -> Result<Json, parity_scale_codec::Error> {
        super::raw_scale_to_json::<V>(raw)
    }
}

impl<S, V> StructOpt for ValueQuery<S, V> {
    fn clap<'a, 'b>() -> App<'a, 'b> {
        App::new("")
    }

    fn from_clap(_: &ArgMatches<'_>) -> Self {
        Self {
            _spook: PhantomData,
        }
    }
}

impl<S, V> AugmentClap for ValueQuery<S, V> {}
