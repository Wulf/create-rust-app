#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
tsync -i $SCRIPT_DIR/**/*.rs $SCRIPT_DIR/**/**/*.rs -o $SCRIPT_DIR/app/src/types/rust.d.ts
