mod chain_spec;

#[paw::main]
fn main(chain: chain_spec::Chain) {
    let spec = chain.generate();
    println!("{}", spec.to_json(true).unwrap());
}
