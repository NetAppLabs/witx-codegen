#!/bin/bash

set -e

witx-codegen \
        --output-type typescript \
        --output-file ../wasm-env/packages/wasi-js/src/wasi_snapshot_preview1_bindings.ts \
        --export-mode \
        --async-mode \
        --error-handler \
        ../go-witx/witx/wasi/wasi_snapshot_preview1.witx
