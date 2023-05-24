#!/usr/bin/env bash

tsc \
    --strict \
    --noUnusedLocals \
    --jsx react-jsx \
    --jsxImportSource preact \
    --allowImportingTsExtensions true \
    --lib es6 \
    --noEmit \
    ./src/*.tsx ./src/*.ts
