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

use anyhow::Result;
use cli::Podman;
use env_logger::Env;
use std::{process::{exit, ExitCode}, path::Path};

pub mod cli;
pub mod podman;
pub mod app;
// pub mod deb;
pub mod app_config;
pub mod defaults;
pub mod deb;
// pub mod fetch;

fn main() -> Result<ExitCode> {
    setup_logger();

    let args = cli::parse();

    match args {
        Podman::Pull { uri } => exit(podman::pull(&uri)),
        Podman::Load { oci } => exit(podman::load(&oci)),
        Podman::Register(reg) => reg.call(),
        Podman::Remove { app: Some(app), .. } => app::remove(Path::new(&app)),
        Podman::Remove { container: Some(container), .. } => podman::purge_container(&container),
        Podman::Remove { .. } => unreachable!(),
        cli::Podman::BuildDeb { oci, app, repo, arch } => {
            exit(deb::ocideb(&oci, &repo, &app, arch.as_deref()));
        }
        Podman::About => {
            println!("Manage podman/oci based flakes;ENGINE");
            Ok(())
        }
    }?;
    Ok(ExitCode::SUCCESS)
}

fn setup_logger() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "info").write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}
