use clap::{AppSettings, Parser, Subcommand, ArgGroup};

/// oci-ctl - Load and Register OCI applications
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Load container
    Load {
        /// OCI container to load into local podman registry
        #[clap(long)]
        oci: String,
    },
    /// Remove application registration or entire container
    #[clap(group(
        ArgGroup::new("remove").required(true).args(&["container", "app"]),
    ))]
    Remove {
        /// Remove all applications registered with the given
        /// container and also remove the container from the
        /// local podman registry
        #[clap(long)]
        container: Option<String>,

        /// Application absolute path to be removed from host
        #[clap(long)]
        app: Option<String>,
    },
    /// Register container application
    Register {
        /// A container name. The name must match with a
        /// name in the local podman registry
        #[clap(long)]
        container: String,

        /// An absolute path to the application inside the container.
        /// If not specified via the target option, the
        /// application will be registered with that path on the
        /// host.
        #[clap(long)]
        app: String,

        /// An absolute path to the application on the host.
        /// Use this option if the application path on the host
        /// should be different to the application path inside
        /// of the container.
        #[clap(long)]
        target: Option<String>,
    },
    /// Build container package
    Build {
        /// OCI container to load into local podman registry
        #[clap(long)]
        oci: String,

        /// An absolute path to the application for registration
        /// at install time of the package.
        #[clap(long, multiple = true)]
        app: Vec<String>,

        /// Output directory to store package(s) as
        /// local debian repository
        #[clap(long)]
        repo: String,
    }
}

pub fn parse_args() -> Cli {
    return Cli::parse();
}
