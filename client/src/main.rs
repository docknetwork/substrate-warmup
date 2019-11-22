mod query;
mod storage_query;

use core::fmt::Debug;
use futures::{
    compat::{Compat, Future01CompatExt},
    future::FutureExt,
};
use jsonrpc_client_transports::transports::ws;
use jsonrpc_client_transports::RpcError;
use serde_json::Value;
use structopt::StructOpt;
use substrate_primitives_storage::StorageData;
use substrate_rpc_api::state::StateClient;
use url::Url;

type BlockHash = <node_template_runtime::Runtime as srml_system::Trait>::Hash;

#[derive(StructOpt, Debug)]
struct Args {
    address: Url,
    #[structopt(flatten)]
    action: Action,
}

#[derive(StructOpt, Debug)]
enum Action {
    Read(query::Key),
}

#[paw::main]
fn main(args: Args) {
    eprintln!("{:#?}", &args);
    let to_print: Value = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(Compat::new(amain(args).boxed()))
        .unwrap();
    println!("{}", serde_json::to_string(&to_print).unwrap());
}

async fn amain(args: Args) -> Result<Value, RpcError> {
    let conn = ws::connect(&args.address.as_str().parse().unwrap())
        .compat()
        .await?;
    let cl = StateClient::<BlockHash>::new(conn);
    let opt: Option<Value> = do_action(cl, args.action).await?;
    Ok(serde_json::to_value(opt).unwrap())
}

async fn do_action(cl: StateClient<BlockHash>, act: Action) -> Result<Option<Value>, RpcError> {
    match act {
        Action::Read(key) => {
            let raw_key = key.to_raw_key();
            let raw_value_opt: Option<StorageData> = cl.storage(raw_key, None).compat().await?;
            raw_value_opt
                .map(|raw_value| key.raw_scale_to_json(raw_value))
                .transpose()
                .map_err(|e| RpcError::Other(e.into()))
        }
    }
}
