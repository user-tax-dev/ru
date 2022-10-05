#!/usr/bin/env bash

_DIR=$(dirname $(realpath "$0"))

cd $_DIR

. ./sh/pid.sh

set -ex

if ! hash watchexec 2>/dev/null; then
cargo install watchexec-cli
fi

if [ ! $cmd ];then
cmd=run
fi

if [ $1 ];then
project=$1
else
project=ru
fi

RUST_BACKTRACE=1 exec watchexec \
  --shell=none -w $project \
  -c -r --exts rs,toml,coffee \
  --ignore target/ \
  -- ./run.sh $project
