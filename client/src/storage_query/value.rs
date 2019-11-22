use crate::storage_query::StorageQuery;
use core::fmt::Debug;
use core::marker::PhantomData;
use parity_scale_codec::FullCodec;
use serde::Serialize;
use srml_support::storage::generator::StorageValue;
use structopt::clap::{App, ArgMatches};
use structopt::StructOpt;
use substrate_primitives_storage::StorageKey;

pub struct ValueQuery<SV: StorageValue<T>, T: FullCodec> {
    _spook: PhantomData<(SV, T)>,
}

impl<SV: StorageValue<T>, T: FullCodec> Debug for ValueQuery<SV, T> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(&(), fmt)
    }
}

impl<SV: StorageValue<T>, T: FullCodec + Serialize> StorageQuery for ValueQuery<SV, T> {
    type Return = T;

    fn to_raw_key(&self) -> StorageKey {
        StorageKey(SV::storage_value_final_key().to_vec())
    }
}

impl<SV: StorageValue<T>, T: FullCodec> StructOpt for ValueQuery<SV, T> {
    fn clap<'a, 'b>() -> App<'a, 'b> {
        App::new("")
    }

    fn from_clap(_: &ArgMatches<'_>) -> Self {
        Self {
            _spook: PhantomData,
        }
    }
}

impl<SV: StorageValue<T>, T: FullCodec> ValueQuery<SV, T> {
    pub fn augment_clap<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app
    }

    pub fn is_subcommand() -> bool {
        false
    }
}
