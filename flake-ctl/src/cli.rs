//
// Copyright (c) 2022 Elektrobit Automotive GmbH
//
// This file is part of flake-pilot
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
use clap::{AppSettings, Parser, Subcommand, ArgGroup};

/// flake-ctl - Manage Flake Applications
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
    /// Load and register OCI applications
    Podman {
        #[clap(subcommand)]
        command: Podman,
    },
    Firecracker {
        #[clap(subcommand)]
        command: Firecracker,
    },
    /// List registered flake applications
    List {
    }
}

#[derive(Subcommand)]
pub enum Firecracker {
    /// Pull image
    #[clap(
        group(
            ArgGroup::new("pull")
                .required(false).args(&["force"])
        ),
        group(
            ArgGroup::new("pullkis")
                .required(false).args(&["kis-image"])
                .conflicts_with("rootfs")
                .conflicts_with("kernel")
                .conflicts_with("initrd")
        ),
        group(
            ArgGroup::new("pullrootfs")
                .required(false).args(&["rootfs", "kernel"])
                .multiple(true)
                .conflicts_with("kis-image")
        ),
        group(
            ArgGroup::new("action")
                .required(true).args(&["kis-image", "rootfs", "kernel"])
                .multiple(true)
        ),
    )]
    Pull {
        /// Image name used as local identifier
        #[clap(long)]
        name: String,

        /// Firecracker image built by KIWI as kis image type
        /// to pull into local image store
        #[clap(long)]
        kis_image: Option<String>,

        /// Single rootfs image to pull into local image store
        #[clap(long, requires = "kernel")]
        rootfs: Option<String>,

        /// Single kernel image to pull into local image store
        #[clap(long, requires = "rootfs")]
        kernel: Option<String>,

        /// Single initrd image to pull into local image store
        #[clap(long)]
        initrd: Option<String>,

        /// Force pulling the image even if it already exists
        /// This will wipe existing data for the provided
        /// identifier
        #[clap(long)]
        force: bool,
    },
    /// Register VM application
    #[clap(
        group(
            ArgGroup::new("register")
                .required(false).args(&["no-net"])
        )
    )]
    Register {
        /// A virtual machine name. The name must match with a
        /// name in the local firecracker registry
        #[clap(long)]
        vm: String,

        /// An absolute path to the application on the host.
        /// If not specified via the target option, the
        /// application will be called with that path inside
        /// of the VM.
        #[clap(long)]
        app: String,

        /// An absolute path to the application in the VM.
        /// Use this option if the application path on the host
        /// should be different to the application path inside
        /// of the VM.
        #[clap(long)]
        target: Option<String>,

        /// Name of the user to run firecracker.
        #[clap(long)]
        run_as: Option<String>,

        /// Resume the VM from previous execution.
        /// If the VM is still running, the app will be
        /// executed inside of this VM instance.
        #[clap(long)]
        resume: bool,

        /// Size of overlay write space in bytes.
        /// Optional suffixes: KiB/MiB/GiB/TiB (1024) or KB/MB/GB/TB (1000)
        #[clap(long)]
        overlay_size: Option<String>,

        /// Disable networking
        #[clap(long)]
        no_net: bool
    },
    /// Remove application registration or entire VM
    #[clap(group(
        ArgGroup::new("remove").required(true).args(&["vm", "app"]),
    ))]
    Remove {
        /// Remove all applications registered with the given
        /// VM and also remove the VM from the
        /// local firecracker registry
        #[clap(long)]
        vm: Option<String>,

        /// Application absolute path to be removed from host
        #[clap(long)]
        app: Option<String>,
    },
}

#[derive(Subcommand)]
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
        #[clap(long, multiple = true)]
        layer: Option<Vec<String>>,

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
