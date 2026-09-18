#!/usr/bin/env bash
# Sandbox icinde hizli derleme kontrolu (Windows gnu hedefi).
set -e
export PATH=/opt/cargo/bin:$PATH
cd "$(dirname "$0")"
echo "== cargo check (x86_64-pc-windows-gnu) =="
cargo check --target x86_64-pc-windows-gnu
