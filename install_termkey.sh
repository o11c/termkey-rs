#!/bin/bash -e
if pkg-config --exists termkey
then
    exit 0
fi

if test -e unibilium || test -e termkey-c
then
    echo 'Error: package not installed, but residue found' >&2
    exit 1
fi

. ./install.conf.sh
make="${make} PREFIX=${prefix}"

git clone --depth 1 git://github.com/mauke/unibilium.git
${make} -j2 -C unibilium
${make} -j2 -C unibilium test
${sudo} ${make} -j2 -C unibilium install
${sudo} ${ldconfig}

git clone --depth 1 git://github.com/o11c/libtermkey.git -b o11c termkey-c
${make} -j2 -C termkey-c
${make} -j2 -C termkey-c test
${sudo} ${make} -j2 -C termkey-c install
${sudo} ${ldconfig}
