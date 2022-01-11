#!/bin/sh

cd `dirname "$0"` || exit 1
exec cargo run -- posts --config config.yaml --output-dir out --force-rebuild
