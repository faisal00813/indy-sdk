#!/usr/bin/env bash


# Rust package cross compile flag
export PKG_CONFIG_ALLOW_CROSS=1
export CARGO_INCREMENTAL=1
export RUST_LOG=indy=trace
export RUST_TEST_THREADS=1
export RUST_BACKTRACE=1

if [ -d "${HOME}/.NDK_TOOLCHAINS" ]; then
    export NDK_TOOLCHAIN_DIR=${HOME}/.NDK_TOOLCHAINS
fi


if [[ -z "${NDK_TOOLCHAIN_DIR}"  ]]; then
    echo "NDK_TOOLCHAIN_DIR is not set. Exiting.... "
    echo "If you have not setup Toolchains then try running install_toolchains.sh."
    exit 1
fi



if [ "$1" == "aarm64" ]; then
    echo "Building for aarch64-linux-android"

    #setup paths for deps
    export SODIUM_LIB_DIR=/usr/local/Cellar/libsodium/1.0.16/lib
    export OPENSSL_DIR=/usr/local/Cellar/openssl/1.0.2l
    export AR=${NDK_TOOLCHAIN_DIR}/arm64/bin/aarch64-linux-android-ar
    export CC=${NDK_TOOLCHAIN_DIR}/arm64/bin/aarch64-linux-android-clang

    # build commands
    cargo clean --target aarch64-linux-android
    cargo build --target aarch64-linux-android --verbose

elif [ "$1" == "armv7" ]; then
    echo "Building for armv7-linux-androideabi"
    export ANDROID_PREBUILT_BINARIES=/tmp/prebuilt_deps_arm
    #cleanup
    rm -rf ${ANDROID_PREBUILT_BINARIES}

    #Download prebuilt deps
    mkdir ${ANDROID_PREBUILT_BINARIES}
    pushd ${ANDROID_PREBUILT_BINARIES}
    curl -L -o libsodium_1.0.16.zip https://repo.sovrin.org/test/sdk-prebuilt-deps/android/deps/armv7/libsodium_1.0.16.zip
    curl -L -o libzmq_4.2.4.zip https://repo.sovrin.org/test/sdk-prebuilt-deps/android/deps/armv7/libzmq_4.2.4.zip
    curl -L -o openssl_1.1.0c.zip https://repo.sovrin.org/test/sdk-prebuilt-deps/android/deps/armv7/openssl_1.1.0c.zip

    #extract deps
    unzip libsodium_1.0.16.zip
    unzip libzmq_4.2.4.zip
    unzip openssl_1.1.0c.zip
    popd
    

    #setup paths for deps
    export SODIUM_LIB_DIR=${ANDROID_PREBUILT_BINARIES}/libsodium_1.0.16/lib
    export LIBZMQ_PREFIX=${ANDROID_PREBUILT_BINARIES}/libzmq_4.2.4
    export OPENSSL_DIR=${ANDROID_PREBUILT_BINARIES}/openssl_1.1.0c
    export AR=${NDK_TOOLCHAIN_DIR}/arm/bin/arm-linux-androideabi-ar
    export CC=${NDK_TOOLCHAIN_DIR}/arm/bin/arm-linux-androideabi-clang

    # build commands
    printenv
    cargo clean --target armv7-linux-androideabi
    cargo build --target armv7-linux-androideabi --verbose
    
elif [ "$1" == "x86" ]; then
    #TODO
    echo "x86 architecture is not supported as of now."
else
    echo "No target architecture provided. Use one of aarm64, armv7 or x86. E.g sh android_build.sh aarm64"
fi

