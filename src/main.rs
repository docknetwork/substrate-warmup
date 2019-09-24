mod chain_spec;

#[derive(structopt::StructOpt)]
/// generate a substrate chainspec
enum Chain {
    /// Prints the chainspec for a shared testnet with bddap as validator
    Ent,
    /// Prints the chainspec for a testnet with Alice as validator
    Ved,
}

#[paw::main]
fn main(chain: Chain) {
    let spec = chain_spec::generate(chain);
    println!("{}", spec.to_json(true).unwrap());
}
