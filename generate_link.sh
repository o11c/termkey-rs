#!/bin/sh -e
echo '#[link_args = "'$(pkg-config --libs termkey) $(pkg-config --libs-only-L termkey | sed 's/-L/-Wl,-rpath=/g')'"] extern {}' > src/generated_link.rs
