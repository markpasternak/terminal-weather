#!/bin/bash
sed -i 's/-> String {/-> \&'\''static str {/g' src/ui/widgets/daily/summary.rs
sed -i 's/\.to_string()//g' src/ui/widgets/daily/summary.rs
cargo clippy
