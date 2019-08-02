use crate::chain_spec;
use crate::service;
use futures::{future, sync::oneshot, Future};
use log::info;
use runtime::GenesisConfig;
use std::cell::RefCell;
pub use substrate_cli::{error, IntoExit, VersionInfo};
use substrate_cli::{parse_and_execute, NoCustom};
use substrate_service::ChainSpec;
use substrate_service::ServiceFactory;
use tokio::runtime::Runtime;

/// Parse command line arguments into service configuration.
pub fn run(args: impl Iterator<Item = String>, version: VersionInfo) -> Result<(), ()> {
    parse_and_execute::<service::Factory, NoCustom, NoCustom, _, _, _, _, _>(
        load_spec,
        &version,
        "substrate-node",
        args,
        Exit,
        |_exit, _cli_args, _custom_args, config| {
            info!("{}", version.name);
            info!("  version {}", config.full_version());
            info!("  by {}, 2017, 2018", version.author);
            info!("Chain specification: {}", config.chain_spec.name());
            info!("Node name: {}", config.name);
            info!("Roles: {:?}", config.roles);
            let mut runtime = Runtime::new().expect("Could not spawn tokio runtime.");
            if config.roles.is_light() {
                runtime.block_on(service::Factory::new_light(config).map_err(|e| format!("{}", e))?)
            } else {
                runtime.block_on(service::Factory::new_full(config).map_err(|e| format!("{}", e))?)
            }
            .map_err(|e| format!("{:?}", e))
        },
    )
    .map_err(|e| log::error!("{:?}", e))
    .map(|v| info!("{:?}", v))
}

fn load_spec(id: &str) -> Result<Option<ChainSpec<GenesisConfig>>, String> {
    match id {
        "dev" => Ok(Some(chain_spec::dev())),
        "local" | "" => Ok(Some(chain_spec::local())),
        _ => Err(format!("\"{}\" is not a supported chain", id)),
    }
}

// A type that can be converted to a future. The substrate chain will shut down when the future
// completes.
struct Exit;

impl IntoExit for Exit {
    type Exit = future::MapErr<oneshot::Receiver<()>, fn(oneshot::Canceled) -> ()>;

    fn into_exit(self) -> Self::Exit {
        // can't use signal directly here because CtrlC takes only `Fn`.
        let (exit_send, exit) = oneshot::channel();

        let exit_send_cell = RefCell::new(Some(exit_send));
        ctrlc::set_handler(move || {
            exit_send_cell
                .replace(None)
                .expect("Exit signal received twice.")
                .send(())
                .expect("Error sending exit notification.")
        })
        .expect("Error setting Ctrl-C handler");

        exit.map_err(drop)
    }
}
