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
extern crate yaml_rust;

use std::env;
use which::which;
use std::path::Path;

pub fn program_abs_path() -> String {
    /*!
    Lookup absolute program path on the filesystem from
    the argv binary name of the caller
    !*/
    let args: Vec<String> = env::args().collect();
    let mut program_path = String::new();
    program_path.push_str(which(&args[0]).unwrap().to_str().unwrap());
    program_path
}

pub fn basename(program_path: &String) -> String {
    /*!
    Get basename from given program path
    !*/
    let mut program_name = String::new();
    program_name.push_str(
        Path::new(program_path).file_name().unwrap().to_str().unwrap()
    );
    program_name
}
