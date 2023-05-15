#!/bin/sh

# Start systemd network and resolver inside of the initrd such
# that we don't require systemd in the main system which allows
# non systemd PID 1 processes like sci do gain network access
#
# For network configuration see the ip= setup from man dracut.cmdline
#
# shellcheck shell=bash
systemctl restart network
systemctl restart systemd-resolved
