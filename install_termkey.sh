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

git clone --depth 1 git://github.com/mauke/unibilium.git
${make} -j2 -C unibilium PREFIX=${prefix}
${sudo} ${make} -j2 -C unibilium install PREFIX=${prefix}
${sudo} ${ldconfig}
git clone --depth 1 git://github.com/o11c/libtermkey.git -b o11c termkey-c
${make} -j2 -C termkey-c PREFIX=${prefix}
${make} -j2 -C termkey-c test PREFIX=${prefix}
${sudo} ${make} -j2 -C termkey-c install PREFIX=${prefix}
${sudo} ${ldconfig}
