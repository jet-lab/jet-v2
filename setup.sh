#!/bin/sh 
if ! command -v cargo watch &> /dev/null
then
    cargo install cargo-watch
else
    echo watch exists 🚢
fi

if ! command -v wasm-pack &> /dev/null
then
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
else
    echo wasm-pack exists ⚡
fi