#!/bin/sh
if pkg-config --exists termkey
then
    exit 0
fi

echo 'Install termkey NYI'
exit 1
