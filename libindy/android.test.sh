#!/usr/bin/env bash



WORKDIR=${PWD}
INDY_DIR="$(realpath "${WORKDIR}/..")"
CI_DIR="${INDY_WORKDIR}/libindy/ci"
ANDROID_BUILD_FOLDER="$(realpath "${WORKDIR}/../android_build")"

TARGET_ARCH=$1

if [ -z "${TARGET_ARCH}" ]; then
    echo STDERR "Missing TARGET_ARCH argument"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi

source ${CI_DIR}/setup.android.env.sh
generate_arch_flags ${TARGET_ARCH}

echo ">> in runner script"
WORKDIR=${PWD}
declare -a EXE_ARRAY

build_test_artifacts(){
    pushd ${WORKDIR}

        cargo clean
        EXE_ARRAY=($( RUSTFLAGS="-L${TOOLCHAIN_DIR}/sysroot/usr/lib -lz -L${TOOLCHAIN_DIR}/${TRIPLET}/lib -L${LIBZMQ_LIB_DIR} -L${SODIUM_LIB_DIR} -lsodium -lzmq -lgnustl_shared" \
                    cargo test --target=${TRIPLET} --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]"))
    popd
}


execute_on_device(){
    #exits the script with code 1 if the emulator is not running
    check_if_emulator_is_running

    adb push \
    "${TOOLCHAIN_DIR}/${TRIPLET}/lib/libgnustl_shared.so" "/data/local/tmp/libgnustl_shared.so"

    adb push \
    "${SODIUM_LIB_DIR}/libsodium.so" "/data/local/tmp/libsodium.so"

    adb push \
    "${LIBZMQ_LIB_DIR}/libzmq.so" "/data/local/tmp/libzmq.so"

    for i in "${EXE_ARRAY[@]}"
    do
       :
        EXE="${i}"
        EXE_NAME=`basename ${EXE}`


        adb push "$EXE" "/data/local/tmp/$EXE_NAME"
        adb shell "chmod 755 /data/local/tmp/$EXE_NAME"
        OUT="$(mktemp)"
        MARK="ADB_SUCCESS!"
        adb shell "LD_LIBRARY_PATH=/data/local/tmp RUST_TEST_THREADS=1 RUST_LOG=debug /data/local/tmp/$EXE_NAME && echo $MARK" 2>&1 | tee $OUT
        grep $MARK $OUT
    done

}



download_sdk
download_and_unzip_dependencies_for_all_architectures
download_and_setup_toolchain
set_env_vars
create_standalone_toolchain_and_rust_target
create_cargo_config
build_test_artifacts
execute_on_device
kill_avd
