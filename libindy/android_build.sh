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

export ANDROID_PREBUILT_BINARIES=/tmp/prebuilt_deps_arm
    
LIBSODIUM=libsodium_1.0.12
LIBZMQ=libzmq_4.2.2
OPENSSL=openssl_1.1.0c
LIBZ=libz_1.2.11


##cleanup
 rm -rf ${ANDROID_PREBUILT_BINARIES}

#Download prebuilt deps
mkdir ${ANDROID_PREBUILT_BINARIES}
pushd ${ANDROID_PREBUILT_BINARIES}
curl -L -o $LIBSODIUM.zip https://repo.sovrin.org/test/sdk-prebuilt-deps/android/deps/armv7/$LIBSODIUM.zip
curl -L -o $LIBZMQ.zip https://repo.sovrin.org/test/sdk-prebuilt-deps/android/deps/armv7/$LIBZMQ.zip
curl -L -o $OPENSSL.zip https://repo.sovrin.org/test/sdk-prebuilt-deps/android/deps/armv7/$OPENSSL.zip
curl -L -o $LIBZ.zip https://repo.sovrin.org/test/sdk-prebuilt-deps/android/deps/armv7/$LIBZ.zip

# #extract deps
unzip -qq $LIBSODIUM.zip
unzip -qq $LIBZMQ.zip
unzip -qq $OPENSSL.zip
unzip -qq $LIBZ.zip
popd

#setup paths for deps
export SODIUM_LIB_DIR=${ANDROID_PREBUILT_BINARIES}/$LIBSODIUM/lib
export LIBZMQ_PREFIX=${ANDROID_PREBUILT_BINARIES}/arm-linux-androideabi-4.9
export OPENSSL_DIR=${ANDROID_PREBUILT_BINARIES}/$OPENSSL
export Z_DIR=${ANDROID_PREBUILT_BINARIES}/android-armeabi-v7a

if [ "$1" == "aarm64" ]; then
    echo "arm64 architecture is not supported as of now."
    #TODO Test arm64 build system
#    echo "Building for aarch64-linux-android"
#    export AR=${NDK_TOOLCHAIN_DIR}/arm64/bin/aarch64-linux-android-ar
#    export CC=${NDK_TOOLCHAIN_DIR}/arm64/bin/aarch64-linux-android-clang
#
#    # build commands
#    cargo clean --target aarch64-linux-android
#    cargo build --target aarch64-linux-android --verbose --release

elif [ "$1" == "armv7" ]; then
    echo "Building for armv7-linux-androideabi"
    export AR=${NDK_TOOLCHAIN_DIR}/arm/bin/arm-linux-androideabi-ar
    export CC=${NDK_TOOLCHAIN_DIR}/arm/bin/arm-linux-androideabi-clang

    printenv

    if [ "$2" == "test" ]; then
        cargo clean --target armv7-linux-androideabi
        cargo test --target armv7-linux-androideabi --no-run --verbose
    else

#        cargo clean --target armv7-linux-androideabi
        cargo build --target armv7-linux-androideabi --verbose #--release
    fi
    



    
elif [ "$1" == "x86" ]; then
    #TODO
    echo "x86 architecture is not supported as of now."
else
    echo "No target architecture provided. Use one of aarm64, armv7 or x86. E.g sh android_build.sh aarm64"
fi

