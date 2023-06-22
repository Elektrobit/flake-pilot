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

pub const SWITCH_ROOT: &str =
    "/sbin/switch_root";
pub const PIVOT_ROOT: &str =
    "/sbin/pivot_root";
pub const OVERLAY_ROOT: &str =
    "/overlayroot/rootfs";
pub const OVERLAY_UPPER: &str =
    "/overlayroot/rootfs_upper";
pub const OVERLAY_WORK: &str =
    "/overlayroot/rootfs_work";
pub const PROBE_MODULE: &str =
    "/sbin/modprobe";
pub const SYSTEMD_NETWORK_RESOLV_CONF: &str =
    "/run/systemd/resolve/resolv.conf";
pub const VM_QUIT: &str =
    "sci_quit";
pub const VHOST_TRANSPORT: &str =
    "vmw_vsock_virtio_transport";
pub const SOCAT: &str =
    "/usr/bin/socat";
pub const VM_PORT: u32 = 
    52;
pub const GUEST_CID: u32 =
    3;

pub fn debug(message: &str) {
    match env::var("PILOT_DEBUG") {
        Ok(_) => { debug!("{}", message) },
        Err(_) => { }
    };
}
