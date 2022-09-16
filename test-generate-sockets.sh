#!/bin/bash

set -e

echo "typescript: /tmp/bindings.ts"
witx-codegen \
        --output-type typescript \
        --output-file /tmp/bindings.ts \
        --export-mode \
        --async-mode \
        ../go-witx/witx/wasi_experimental_sockets.witx

echo "rust: /tmp/bindings.rs"
witx-codegen \
        --output-type rust \
        --output-file /tmp/bindings.rs \
        ../go-witx/witx/wasi_experimental_sockets.witx

echo "zig: /tmp/bindings.zig"
witx-codegen \
        --output-type zig \
        --output-file /tmp/bindings.rs \
        ../go-witx/witx/wasi_experimental_sockets.witx
