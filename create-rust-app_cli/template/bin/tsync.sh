#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
tsync -i ../backend -o $SCRIPT_DIR/../frontend/src/types/rust.d.ts
