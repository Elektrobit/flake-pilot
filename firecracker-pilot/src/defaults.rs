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
use std::env;

pub const IMAGE_ROOT: &str =
    "image";
pub const IMAGE_OVERLAY: &str =
    "overlayroot";
pub const OVERLAY_ROOT: &str =
    "overlayroot/rootfs";
pub const OVERLAY_UPPER: &str =
    "overlayroot/rootfs_upper";
pub const OVERLAY_WORK: &str =
    "overlayroot/rootfs_work";
pub const FIRECRACKER_OVERLAY_DIR:&str =
    "/var/lib/firecracker/storage";
pub const FIRECRACKER_TEMPLATE:&str =
    "/etc/flakes/firecracker.json";
pub const FIRECRACKER_FLAKE_DIR: &str =
    "/usr/share/flakes";
pub const FIRECRACKER_VMID_DIR: &str =
    "/var/lib/firecracker/storage/tmp/flakes";
pub const GC_THRESHOLD: i32 = 20;
pub const VM_CID: u32 = 3;
pub const VM_PORT: u32 =
    52;
pub const SOCAT: &str =
    "/usr/bin/socat";
pub const RETRIES: u32 =
    60;
pub const VM_WAIT_TIMEOUT_MSEC: u64 =
    1000;

pub fn is_debug() -> bool {
    let debug_set;
    match env::var("PILOT_DEBUG") {
        Ok(_) => { debug_set = true },
        Err(_) => { debug_set = false }
    };
    debug_set
}

pub fn debug(message: &str) {
    if is_debug() {
        debug!("{}", message)
    }
}
