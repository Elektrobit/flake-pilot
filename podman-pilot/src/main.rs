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

#[cfg(test)]
pub mod tests;
pub mod error;

use std::process::{ExitCode, Termination};

use config::config;
use env_logger::Env;
use error::FlakeError;

pub mod app_path;
pub mod podman;
pub mod defaults;
pub mod config;

fn main() -> ExitCode {
    setup_logger();
    // load config now so we can terminate early if the config is invalid
    config();
    // past here there should be no more panics

    let result = run();

    // TODO: implement cleanup function 
    // cleanup()

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err}");
            err.report()
        },
    }
}

fn run() -> Result<(), FlakeError> {

    let program_path = app_path::program_abs_path();
    let program_name = app_path::basename(&program_path);

    let container = podman::create(&program_name)?;
    let cid = &container.0;
    podman::start(
        &program_name,
        cid
    )
}

fn setup_logger() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}
