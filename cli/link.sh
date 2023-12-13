#!/bin/bash

set -euf -o pipefail

cargo build --release
cp target/release/cli /usr/local/bin/co2
