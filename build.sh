#!/usr/bin/env bash

DIR=$(dirname $(realpath "$0"))
cd $DIR
set -ex

./sh/gen.init.coffee misc
./sh/gen.init.coffee redis
cd ru
rm -rf lib
nr build
bun run cep -- -c src -o lib
cd $DIR
./sh/gen.init.coffee ru
