#!/usr/bin/env bash
cd "$(dirname "$0")"
cd ..

npm run build-debug
node test.js
