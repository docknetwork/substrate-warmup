// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Custom version of substrate/core/chain-spec/src/chain_spec.rs which allows for dynamic
//! initialization of a serializable chainspec.
//! This file is a workaround for https://github.com/paritytech/substrate/issues/3750.
//! if the issue is resolved, this file can go away.

// This file was copied from the source and modified to make it work.
// It's in a working state but it contains unnessesary code that can be factored out.

use serde::{Deserialize, Serialize};
use serde_json as json;
use sr_primitives::{BuildStorage, ChildrenStorageOverlay, StorageOverlay};
use std::collections::HashMap;
use substrate_primitives::storage::{StorageData, StorageKey};
use substrate_service::RuntimeGenesis;
use substrate_telemetry::TelemetryEndpoints;

#[derive(Clone)]
struct GenesisSource<G>(G);

impl<G: RuntimeGenesis> GenesisSource<G> {
    fn resolve(self) -> Genesis<G> {
        Genesis::Runtime(self.0)
    }
}

impl<G: RuntimeGenesis> BuildStorage for ChainSpec<G> {
    fn build_storage(self) -> Result<(StorageOverlay, ChildrenStorageOverlay), String> {
        match self.genesis.resolve() {
            Genesis::Runtime(gc) => gc.build_storage(),
            Genesis::Raw(map, children_map) => Ok((
                map.into_iter().map(|(k, v)| (k.0, v.0)).collect(),
                children_map
                    .into_iter()
                    .map(|(sk, map)| (sk.0, map.into_iter().map(|(k, v)| (k.0, v.0)).collect()))
                    .collect(),
            )),
        }
    }

    fn assimilate_storage(
        self,
        _: &mut (StorageOverlay, ChildrenStorageOverlay),
    ) -> Result<(), String> {
        Err("`assimilate_storage` not implemented for `ChainSpec`.".into())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
enum Genesis<G> {
    Runtime(G),
    Raw(
        HashMap<StorageKey, StorageData>,
        HashMap<StorageKey, HashMap<StorageKey, StorageData>>,
    ),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ChainSpecFile {
    pub name: String,
    pub id: String,
    pub boot_nodes: Vec<String>,
    pub telemetry_endpoints: Option<TelemetryEndpoints>,
    pub protocol_id: Option<String>,
    pub consensus_engine: Option<String>,
    pub properties: Option<Properties>,
}

/// Arbitrary properties defined in chain spec as a JSON object
pub type Properties = json::map::Map<String, json::Value>;

/// A configuration of a chain. Can be used to build a genesis block.
#[derive(Clone)]
pub struct ChainSpec<G> {
    spec: ChainSpecFile,
    genesis: GenesisSource<G>,
}

impl<G> ChainSpec<G> {
    /// Create hardcoded spec.
    pub fn from_genesis(
        name: &str,
        id: &str,
        genesis: G,
        boot_nodes: Vec<String>,
        telemetry_endpoints: Option<TelemetryEndpoints>,
        protocol_id: Option<&str>,
        consensus_engine: Option<&str>,
        properties: Option<Properties>,
    ) -> Self {
        let spec = ChainSpecFile {
            name: name.to_owned(),
            id: id.to_owned(),
            boot_nodes,
            telemetry_endpoints,
            protocol_id: protocol_id.map(str::to_owned),
            consensus_engine: consensus_engine.map(str::to_owned),
            properties,
        };
        ChainSpec {
            spec,
            genesis: GenesisSource(genesis),
        }
    }

    pub fn protocol_id(&self) -> Option<&str> {
        self.spec.protocol_id.as_ref().map(|x| &**x)
    }
}

impl<G: RuntimeGenesis> Into<substrate_service::ChainSpec<G>> for ChainSpec<G> {
    // Unfortunately we need to serialize then deserialize to do this conversion because of
    // https://github.com/paritytech/substrate/issues/3750
    fn into(self) -> substrate_service::ChainSpec<G> {
        let json = self
            .into_json(false)
            .expect("error serializing chainspec while converting")
            .into_bytes();
        substrate_service::ChainSpec::from_json_bytes(json)
            .expect("error serializing chainspec while converting")
    }
}

impl<G: RuntimeGenesis> ChainSpec<G> {
    /// Dump to json string.
    pub fn into_json(self, raw: bool) -> Result<String, String> {
        #[derive(Serialize, Deserialize)]
        struct Container<G> {
            #[serde(flatten)]
            spec: ChainSpecFile,
            genesis: Genesis<G>,
        };
        let genesis = match (raw, self.genesis.resolve()) {
            (true, Genesis::Runtime(g)) => {
                let storage = g.build_storage()?;
                let top = storage
                    .0
                    .into_iter()
                    .map(|(k, v)| (StorageKey(k), StorageData(v)))
                    .collect();
                let children = storage
                    .1
                    .into_iter()
                    .map(|(sk, child)| {
                        (
                            StorageKey(sk),
                            child
                                .into_iter()
                                .map(|(k, v)| (StorageKey(k), StorageData(v)))
                                .collect(),
                        )
                    })
                    .collect();

                Genesis::Raw(top, children)
            }
            (_, genesis) => genesis,
        };
        let spec = Container {
            spec: self.spec,
            genesis,
        };
        json::to_string_pretty(&spec).map_err(|e| format!("Error generating spec json: {}", e))
    }
}
