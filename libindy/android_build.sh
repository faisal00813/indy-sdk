#!/usr/bin/env bash

# Rust package cross compile flag
export PKG_CONFIG_ALLOW_CROSS=1

if [ -d "${HOME}/.NDK_TOOLCHAINS" ]; then
   export NDK_TOOLCHAIN_DIR=${HOME}/.NDK_TOOLCHAINS
fi

if [[ -z "${NDK_TOOLCHAIN_DIR}"  ]]; then
    echo "NDK_TOOLCHAIN_DIR is not set. Exiting.... "
    echo "If you have not setup Toolchains then tru running install_toolchains.sh."
    exit 1
fi
echo "NDK dir : ${NDK_TOOLCHAIN_DIR}"

if [ "$1" == "aarm64" ]; then
    echo "Building for aarch64-linux-android"
    # Libsodium deps resolutuon
    #export SODIUM_LIB_DIR=/usr/local/Cellar/libsodium/1.0.16/lib

    #Openssl deps resolution
    export OPENSSL_DIR=/usr/local/Cellar/openssl/1.0.2l
    export AR=${NDK_TOOLCHAIN_DIR}/arm64/bin/aarch64-linux-android-ar
    export CC=${NDK_TOOLCHAIN_DIR}/arm64/bin/aarch64-linux-android-clang
    # build commands
    cargo clean --target aarch64-linux-android
    cargo build --target aarch64-linux-android --verbose --release
elif [ "$1" == "armv7" ]; then
    echo "Building for armv7-linux-androideabi"
    #Download prebuilt deps
    # if [ -f indy-sdk-prebuilt-deps.zip ]; then
        # tar indy-sdk-prebuilt-deps.zip
    # else 
        # curl -L -o indy-sdk-prebuilt-deps.zip https://drive.google.com/uc?id=1tEqN8z6B6N8AOQgAb8g-Vj7DmyvRGTc6
        # tar indy-sdk-prebuilt-deps.zip
    # fi
    pushd /tmp
    unzip /tmp/prebuilt_deps.zip
    pushd prebuilt_deps/android/deps/armv7
    unzip libsodium_1.0.16.zip
    unzip libzmq_4.2.4.zip
    unzip openssl_1.1.0c.zip
    popd 
    popd 
    export ANDROID_PREBUILT_BINARIES=/tmp/prebuilt_deps/android/deps/armv7
    export SODIUM_LIB_DIR=${ANDROID_PREBUILT_BINARIES}/libsodium_1.0.16/lib
    export LIBZMQ_PREFIX=${ANDROID_PREBUILT_BINARIES}/libzmq_4.2.4
    export OPENSSL_DIR=${ANDROID_PREBUILT_BINARIES}/openssl_1.1.0c
    # export ZMQPW_DIR=/Users/abdussami/Work/libs/libzmq-android-bins
    export AR=${NDK_TOOLCHAIN_DIR}/arm/bin/arm-linux-androideabi-ar
    export CC=${NDK_TOOLCHAIN_DIR}/arm/bin/arm-linux-androideabi-clang
    # build commands
    cargo clean --target armv7-linux-androideabi
    cargo build --target armv7-linux-androideabi --verbose --release
elif [ "$1" == "x86" ]; then
    #TODO
    echo "x86 architecture is not supported as of now."
else
    echo "No target architecture provided. Use one of aarm64, armv7 or x86. E.g sh android_build.sh aarm64"
fi

#cleanup

unset AR
unset CC
unset ANDROID_PREBUILT_BINARIES
