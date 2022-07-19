pub mod cli;
pub mod podman;
pub mod app;

fn main() {
    let args = cli::parse_args();

    match &args.command {
        // load
        cli::Commands::Load { oci } => {
            podman::load(oci);
        },
        // register
        cli::Commands::Register { container, app, target } => {
            app::register(container, app, target.as_ref());
        },
        // remove
        cli::Commands::Remove { container, app } => {
            if ! app.is_none() {
                app::remove(app.as_ref().map(String::as_str).unwrap());
            }
            if ! container.is_none() {
                app::purge(container.as_ref().map(String::as_str).unwrap());
            }
        }
    }
}
