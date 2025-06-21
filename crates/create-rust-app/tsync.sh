#!/bin/bash

tsync -i src/auth -o plugin-auth.d.ts
tsync -i src/dev -o plugin-dev.d.ts
tsync -i src/storage -o plugin-storage.d.ts