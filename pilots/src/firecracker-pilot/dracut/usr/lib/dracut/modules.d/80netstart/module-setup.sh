#!/usr/bin/bash

declare moddir=${moddir}

check() {
    return 255
}

depends() {
    echo network systemd-networkd systemd-resolved
    return 0
}

install() {
    inst_hook pre-pivot 80 "$moddir/netstart.sh"
}
