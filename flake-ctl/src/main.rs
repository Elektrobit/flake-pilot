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
#[macro_use]
extern crate log;

use env_logger::Env;
use std::process::{exit, ExitCode};

pub mod cli;
pub mod podman;
pub mod firecracker;
pub mod app;
pub mod deb;
pub mod app_config;
pub mod defaults;
pub mod fetch;

#[tokio::main]
async fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    setup_logger();

    let args = cli::parse_args();

    match &args.command {
        // list
        cli::Commands::List { } => {
            info!("Registered applications:");
            let app_names = app::app_names();
            if app_names.is_empty() {
                println!("No application(s) registered");
            } else {
                for app in app_names {
                    println!("- {}", app);
                }
            }
        },
        // firecracker engine
        cli::Commands::Firecracker { command } => {
            match &command {
                // pull
                cli::Firecracker::Pull {
                    name, kis_image, rootfs, kernel, initrd, force
                } => {
                    if ! kis_image.is_none() {
                        exit(
                            firecracker::pull_kis_image(
                                name, kis_image.as_ref(), *force
                            ).await
                        );
                    } else {
                        exit(
                            firecracker::pull_component_image(
                                name, rootfs.as_ref(), kernel.as_ref(),
                                initrd.as_ref(), *force
                            ).await
                        );
                    }
                },
                // register
                cli::Firecracker::Register {
                    vm, app, target, run_as, overlay_size, no_net, resume,
                    include_tar
                } => {
                    if app::init(Some(app)) {
                        let mut ok = app::register(
                            Some(app), target.as_ref(),
                            defaults::FIRECRACKER_PILOT
                        );
                        if ok {
                            ok = app::create_vm_config(
                                vm,
                                Some(app),
                                target.as_ref(),
                                run_as.as_ref(),
                                overlay_size.as_ref(),
                                *no_net,
                                *resume,
                                include_tar.as_ref().cloned()
                            );
                        }
                        if ! ok {
                            app::remove(
                                app, defaults::FIRECRACKER_PILOT, true
                            );
                            return Ok(ExitCode::FAILURE)
                        }
                    } else {
                        return Ok(ExitCode::FAILURE)
                    }
                },
                // remove
                cli::Firecracker::Remove { vm, app } => {
                    if ! app.is_none() {
                        app::remove(
                            app.as_ref().map(String::as_str).unwrap(),
                            defaults::FIRECRACKER_PILOT, false
                        );
                    }
                    if ! vm.is_none() {
                        app::purge(
                            vm.as_ref().map(String::as_str).unwrap(),
                            defaults::FIRECRACKER_PILOT
                        );
                    }
                }
            }
        },
        // podman engine
        cli::Commands::Podman { command } => {
            match &command {
                // pull
                cli::Podman::Pull { uri } => {
                    exit(podman::pull(uri));
                },
                // load
                cli::Podman::Load { oci } => {
                    exit(podman::load(oci));
                },
                // register
                cli::Podman::Register {
                    container, app, target, base,
                    layer, include_tar, resume, attach, run_as, opt, info
                } => {
                    if *info {
                        podman::print_container_info(container);
                    } else if app::init(app.as_ref()) {
                        let mut ok = app::register(
                            app.as_ref(), target.as_ref(),
                            defaults::PODMAN_PILOT
                        );
                        if ok {
                            ok = app::create_container_config(
                                container,
                                app.as_ref(),
                                target.as_ref(),
                                base.as_ref(),
                                layer.as_ref().cloned(),
                                include_tar.as_ref().cloned(),
                                *resume,
                                *attach,
                                run_as.as_ref(),
                                opt.as_ref().cloned()
                            );
                        }
                        if ! ok {
                            app::remove(
                                app.as_ref().map(String::as_str).unwrap(),
                                defaults::PODMAN_PILOT, true
                            );
                            return Ok(ExitCode::FAILURE)
                        }
                    } else {
                        return Ok(ExitCode::FAILURE)
                    }
                },
                // remove
                cli::Podman::Remove { container, app } => {
                    if ! app.is_none() {
                        app::remove(
                            app.as_ref().map(String::as_str).unwrap(),
                            defaults::PODMAN_PILOT, false
                        );
                    }
                    if ! container.is_none() {
                        app::purge(
                            container.as_ref().map(String::as_str).unwrap(),
                            defaults::PODMAN_PILOT
                        );
                    }
                }
                // build deb
                cli::Podman::BuildDeb { oci, app, repo, arch } => {
                    exit(deb::ocideb(oci, repo, app, arch.as_ref()));
                }
            }
        },
    }
    Ok(ExitCode::SUCCESS)
}

fn setup_logger() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}
