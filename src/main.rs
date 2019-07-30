mod chain_spec;
mod cli;
mod service;

use substrate_cli::VersionInfo;

fn main() {
    let version = VersionInfo {
        name: "Substrate Node",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "node-template",
        author: "Anonymous",
        description: "Template Node",
        support_url: "support.anonymous.an",
    };

    if let Err(e) = cli::run(::std::env::args(), version) {
        eprintln!("Error starting the node: {:?}", e);
        std::process::exit(1)
    }
}
