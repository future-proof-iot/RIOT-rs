#!/bin/sh

# script is used by github workflows

grep '^channel =' rust-toolchain.toml | sed 's/.*= "\(.*\)"/\1/'
