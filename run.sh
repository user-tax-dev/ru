#!/usr/bin/env bash

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex

cd $1
rm -rf lib
bun run cargo-cp-artifact -- -nc lib/lib.node -- cargo build --features main --message-format=json-render-diagnostics
bun run cep -- -c test -o lib
cd $DIR
./sh/gen.init.coffee $1
$1/lib/test.js

