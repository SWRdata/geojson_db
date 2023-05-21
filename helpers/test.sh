#!/usr/bin/env bash
cd "$(dirname "$0")"
cd ..

#npm run build-debug
npm run build-release
node test.js
