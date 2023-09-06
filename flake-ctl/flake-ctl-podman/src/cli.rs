use std::{fs::OpenOptions, path::Path};

use anyhow::{bail, Context, Result};
use clap::{ArgGroup, Args, Parser};

use crate::{
    app,
    app_config::{AppConfig, AppContainer, AppContainerRuntime, AppInclude},
    defaults, podman,
};

#[derive(Parser)]
pub enum Podman {
    /// Pull container
    Pull {
        /// OCI image to pull from remote registry into local podman registry
        #[clap(long)]
        uri: String,
    },
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
    #[clap(
        group(
            ArgGroup::new("register")
                .required(false).args(&["resume", "attach"])
        ),
        group(
            ArgGroup::new("application")
                .required(true).args(&["app", "info"])
        )
    )]
    Register(Register),
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
    },
    /// Print the info string for flake-ctl
    About
}

#[derive(Debug, Args)]
pub struct Register {
    /// A container name. The name must match with a
    /// name in the local podman registry
    #[clap(long)]
    container: String,

    /// An absolute path to the application on the host.
    /// If not specified via the target option, the
    /// application will be called with that path inside
    /// of the container.
    #[clap(long)]
    app: Option<String>,

    /// An absolute path to the application in the container.
    /// Use this option if the application path on the host
    /// should be different to the application path inside
    /// of the container. Set this option to just "/"
    /// if the default entrypoint of the container should
    /// be called.
    #[clap(long)]
    target: Option<String>,

    /// Name of the base container. The name must match with a
    /// name in the local podman registry
    #[clap(long)]
    base: Option<String>,

    /// Name of an additional container layer on top of
    /// the specified base container. This option can be
    /// specified multiple times. The resulting layer list
    /// is evaluated in the order of the arguments as they
    /// were provided on the command line.
    #[clap(long, multiple = true, alias = "layer")]
    layers: Option<Vec<String>>,

    /// Name of a tar file to be included on top of
    /// the container instance. This option can be
    /// specified multiple times.
    #[clap(long, multiple = true)]
    include_tar: Option<Vec<String>>,

    /// Resume the container from previous execution.
    /// If the container is still running, the app will be
    /// executed inside of this container instance.
    #[clap(long)]
    resume: bool,

    /// Attach to the container if still running, rather than
    /// executing the app again. Only makes sense for interactive
    /// sessions like a shell application.
    #[clap(long)]
    attach: bool,

    /// Name of the user to run podman. Note: This requires
    /// rootless podman to be configured on the host. It's also
    /// important to understand that the user's HOME registry
    /// will be used to lookup the containers. It is not possible
    /// to provision base- or layers of containers across multiple
    /// container registries.
    #[clap(long)]
    run_as: Option<String>,

    /// Container runtime option, and optional value, used to
    /// create the container. This option can be
    /// specified multiple times.
    #[clap(long, multiple = true)]
    opt: Option<Vec<String>>,

    /// Print registration information from container if provided
    #[clap(long)]
    info: bool,
}

impl Register {
    pub fn call(self) -> Result<()> {
        if self.info {
            return podman::print_container_info(&self.container);
        }

        let app = self.app.unwrap();

        if self.base.is_none() && self.layers.is_some() {
            bail!("Layer(s) specified without a base");
        }

        app::init(&app)?;
        app::register(&app, self.target.as_ref().unwrap_or(&app), defaults::PODMAN_PILOT)?;

        let config = AppConfig {
            include: AppInclude { tar: self.include_tar },
            container: AppContainer {
                name: self.container,
                target_app_path: self.target.unwrap_or_else(|| app.clone()),
                host_app_path: app.clone(),
                base_container: self.base,
                layers: self.layers,
                runtime: AppContainerRuntime {
                    runas: self.run_as,
                    resume: Some(self.resume),
                    attach: Some(self.attach),
                    podman: self.opt,
                },
            },
        };

        let base_name = Path::new(&app).file_stem().unwrap_or_default();

        serde_yaml::to_writer(
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(Path::new(defaults::FLAKE_DIR).join(&base_name).with_extension("yaml"))
                .context("Could not open yaml file")?,
            &config,
        )?;

        Ok(())
    }
}

pub(crate) fn parse() -> Podman {
    Podman::parse()
}
