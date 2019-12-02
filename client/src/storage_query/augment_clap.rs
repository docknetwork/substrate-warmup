use structopt::clap::App;

/// Clap secretly requires these methods--augment_clap and is_subcommand.
/// This is a helper to default impl.
pub trait AugmentClap {
    fn augment_clap<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
        app
    }

    fn is_subcommand() -> bool {
        false
    }
}
