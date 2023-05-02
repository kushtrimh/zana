#!/bin/bash

mkdir -p dist.chrome.mv3.build
cp -r extension/addon/* dist.chrome.mv3.build
rm dist.chrome.mv3.build/manifest.json
cp extension/platform/chrome/manifest.json dist.chrome.mv3.build
if [ "$1" = "--release" ]; then
    mkdir -p dist.chrome.mv3.release
    zip -r dist.chrome.mv3.release/extension.zip dist.chrome.mv3.build
fi
