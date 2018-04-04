
set -ex
main() {
#clean /tmp
pushd /tmp

#download zip
curl -L -o prebuilt_deps.zip https://transfer.sh/Li1y9/prebuilt_deps.zip
unzip /tmp/prebuilt_deps.zip
pushd prebuilt_deps/android/deps/armv7
unzip libsodium_1.0.16.zip
unzip libzmq_4.2.4.zip
unzip openssl_1.1.0c.zip
popd
popd
#set env variables
export ANDROID_PREBUILT_BINARIES=/tmp/prebuilt_deps/android/deps/armv7
export SODIUM_LIB_DIR=${ANDROID_PREBUILT_BINARIES}/libsodium_1.0.16/lib
export LIBZMQ_PREFIX=${ANDROID_PREBUILT_BINARIES}/libzmq_4.2.4
export PKG_CONFIG_ALLOW_CROSS=1
export CARGO_INCREMENTAL=1
export RUST_LOG=indy=trace
export RUST_TEST_THREADS=1
export RUST_BACKTRACE=1
printenv
}

main "${@}"