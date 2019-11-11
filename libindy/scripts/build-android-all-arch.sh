#!/usr/bin/env bash

echo "Building for arm"
bash build-android.sh -d arm
echo "Building for arm64"
bash build-android.sh -d arm64
echo "Building for x86"
bash build-android.sh -d x86
