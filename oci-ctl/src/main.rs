//
// Copyright (c) 2022 Elektrobit Automotive GmbH
//
// This file is part of oci-pilot
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
use std::process::exit;

pub mod cli;
pub mod podman;
pub mod app;
pub mod deb;
pub mod app_config;
pub mod defaults;

fn main() {
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
                    layer, include_tar, resume, attach
                } => {
                    if app::init() {
                        app::register(
                            container,
                            app,
                            target.as_ref(),
                            base.as_ref(),
                            layer.as_ref().cloned(),
                            include_tar.as_ref().cloned(),
                            resume.as_ref(),
                            attach.as_ref()
                        );
                    }
                },
                // remove
                cli::Podman::Remove { container, app } => {
                    if ! app.is_none() {
                        app::remove(
                            app.as_ref().map(String::as_str).unwrap()
                        );
                    }
                    if ! container.is_none() {
                        app::purge(
                            container.as_ref().map(String::as_str).unwrap()
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
}

fn setup_logger() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}
