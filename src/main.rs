mod chain_spec;
mod serializable_genesis;

#[paw::main]
fn main(chain: chain_spec::Chain) {
    let spec = chain.generate();
    println!("{}", spec.into_json(true).unwrap());
}
