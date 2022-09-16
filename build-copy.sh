#!/bin/bash
set -x 

cargo build --release
cp target/release/witx-codegen ~/.cargo/bin
