#!/bin/bash
set -ex
cargo doc
cargo test --workspace
cd core_macros
cargo publish
cd ..
cd core
cargo publish
cd ..
cd macros
cargo publish
cd ..
cargo publish
echo "published successfully."
