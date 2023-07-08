#!/bin/bash
# This is a hack and related to the issue explained here:
# https://github.com/rust-lang/rust/issues/99382
#
set -ex

if [ ! -e /usr/bin/sudo ]; then
    echo "no sudo available... skipped"
    exit 0
fi
if [ -e /usr/bin/gcc-11.bin ];then
    echo "gcc already wrapped... skipped"
    exit 0
fi
if [ ! -e /usr/bin/gcc-11 ];then
    echo "no gcc-11 system... skipped"
    exit 0
fi
mv /usr/bin/gcc-11 /usr/bin/gcc-11.bin

cat >/usr/bin/gcc-11 <<- EOF
#!/bin/bash
args=\$(echo \$@ | sed -e "s@static-pie@static@")
/usr/bin/gcc-11.bin \$args
EOF

chmod 755 /usr/bin/gcc-11
