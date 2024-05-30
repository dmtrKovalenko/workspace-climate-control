#!/bin/bash

set -euf -o pipefail

cargo build --release
cp target/release/co2nsole /usr/local/bin/co2