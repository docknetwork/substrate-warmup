use crate::storage_query::StorageQuery;
use core::fmt::Debug;
use core::marker::PhantomData;
use parity_scale_codec::FullCodec;
use serde::Serialize;
use srml_support::storage::generator::StorageMap;
use structopt::clap::{App, ArgMatches};
use structopt::StructOpt;
use substrate_primitives_storage::StorageKey;

pub struct MapQuery<SV: StorageMap<K, V>, K: FullCodec, V: FullCodec> {
    key: K,
    _spook: PhantomData<(SV, V)>,
}

impl<SV: StorageMap<K, V>, K: FullCodec, V: FullCodec> Debug for MapQuery<SV, K, V> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(&(), fmt)
    }
}

impl<SV: StorageMap<K, V>, K: FullCodec, V: FullCodec + Serialize> StorageQuery
    for MapQuery<SV, K, V>
{
    type Return = V;

    fn to_raw_key(&self) -> StorageKey {
        StorageKey(SV::storage_map_final_key(&self.key).as_ref().to_vec())
    }
}

impl<SV: StorageMap<K, V>, K: FullCodec + StructOpt, V: FullCodec> StructOpt
    for MapQuery<SV, K, V>
{
    fn clap<'a, 'b>() -> App<'a, 'b> {
        K::clap()
    }

    fn from_clap(matches: &ArgMatches<'_>) -> Self {
        Self {
            key: K::from_clap(matches),
            _spook: PhantomData,
        }
    }
}

impl<SV: StorageMap<K, V>, K: FullCodec, V: FullCodec> MapQuery<SV, K, V> {
    pub fn augment_clap<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app
    }

    pub fn is_subcommand() -> bool {
        false
    }
}
