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
        /// OCI image to load into local podman registry
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

        /// An absolute path to the application on the host.
        /// If not specified via the target option, the
        /// application will be called with that path inside
        /// of the container.
        #[clap(long)]
        app: String,

        /// An absolute path to the application in the container.
        /// Use this option if the application path on the host
        /// should be different to the application path inside
        /// of the container. Set this option to just "/"
        /// if the default entrypoint of the container should
        /// be called.
        #[clap(long)]
        target: Option<String>,
    },
    /// List registered container applications
    List {
    },
    /// Build container package
    BuildDeb {
        /// OCI image to load into local podman registry
        #[clap(long)]
        oci: String,

        /// An absolute path to the application on the host
        /// and optional absolute path to the application in the
        /// container. The path spec is separated by a semicolon.
        /// This option can be specified multiple times.
        #[clap(long, multiple = true)]
        app: Vec<String>,

        /// Output directory to store package(s) as
        /// local debian repository
        #[clap(long)]
        repo: String,

        /// Package BuildArch architecture
        /// If not specified it will be taken from uname -m
        #[clap(long)]
        arch: Option<String>,
    }
}

pub fn parse_args() -> Cli {
    return Cli::parse();
}
