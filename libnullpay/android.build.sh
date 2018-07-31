#!/usr/bin/env bash
export BLACK=`tput setaf 0`
export RED=`tput setaf 1`
export GREEN=`tput setaf 2`
export YELLOW=`tput setaf 3`
export BLUE=`tput setaf 4`
export MAGENTA=`tput setaf 5`
export CYAN=`tput setaf 6`
export WHITE=`tput setaf 7`

export BOLD=`tput bold`
export RESET=`tput sgr0`

WORKDIR="$( cd "$(dirname "$0")" ; pwd -P )"
CI_DIR="${WORKDIR}/../libindy/ci"
export ANDROID_BUILD_FOLDER="/tmp/android_build"
DOWNLOAD_PREBUILTS="0"

while getopts ":d" opt; do
    case ${opt} in
        d) export DOWNLOAD_PREBUILTS="1";;
        \?);;
    esac
done
shift $((OPTIND -1))

TARGET_ARCH=$1

if [ -z "${TARGET_ARCH}" ]; then
    echo STDERR "${RED}Missing TARGET_ARCH argument${RESET}"
    echo STDERR "${BLUE}e.g. x86 or arm${RESET}"
    exit 1
fi

source ${CI_DIR}/setup.android.env.sh

create_cargo_config(){
mkdir -p ${WORKDIR}/.cargo
cat << EOF > ${WORKDIR}/.cargo/config
[target.${TRIPLET}]
ar = "$(realpath ${AR})"
linker = "$(realpath ${CC})"
EOF
}
setup_dependencies(){
   if [ "${DOWNLOAD_PREBUILTS}" == "1" ]; then
        download_and_unzip_dependencies_for_all_architectures
        else
            echo "${BLUE}Not downloading prebuilt dependencies. Dependencies locations have to be passed${RESET}"
            if [ -z "${OPENSSL_DIR}" ]; then
                OPENSSL_DIR="openssl_${TARGET_ARCH}"
                if [ -d "${OPENSSL_DIR}" ] ; then
                    echo "${GREEN}Found ${OPENSSL_DIR}${RESET}"
                elif [ -z "$3" ]; then
                    echo STDERR "${RED}Missing OPENSSL_DIR argument and environment variable${RESET}"
                    echo STDERR "${BLUE}e.g. set OPENSSL_DIR=<path> for environment or openssl_${TARGET_ARCH}${RESET}"
                    exit 1
                else
                    OPENSSL_DIR=$3
                fi
            fi

            if [ -z "${SODIUM_DIR}" ]; then
                SODIUM_DIR="libsodium_${TARGET_ARCH}"
                if [ -d "${SODIUM_DIR}" ] ; then
                    echo "${GREEN}Found ${SODIUM_DIR}${RESET}"
                elif [ -z "$4" ]; then
                    echo STDERR "${RED}Missing SODIUM_DIR argument and environment variable${RESET}"
                    echo STDERR "${BLUE}e.g. set SODIUM_DIR=<path> for environment or libsodium_${TARGET_ARCH}${RESET}"
                    exit 1
                else
                    SODIUM_DIR=$4
                fi
            fi

    fi

    if [ -z "${INDY_DIR}" ] ; then
            INDY_DIR="libindy_${TARGET_ARCH}"
            if [ -d "${INDY_DIR}" ] ; then
                echo "${GREEN}Found ${INDY_DIR}${RESET}"
            elif [ -z "$2" ] ; then
                echo STDERR "${RED}Missing INDY_DIR argument and environment variable${RESET}"
                echo STDERR "${BLUE}e.g. set INDY_DIR=<path> for environment or libindy_${TARGET_ARCH}${RESET}"
                exit 1
            else
                INDY_DIR=$2
            fi

        if [ -d "${INDY_DIR}/lib" ] ; then
            INDY_DIR="${INDY_DIR}/lib"
        fi
     fi


}


package_library(){
    echo "${GREEN}Packaging in zip file${RESET}"
    mkdir -p ${ANDROID_BUILD_FOLDER}/libnullpay_${TARGET_ARCH}/include
    mkdir -p ${ANDROID_BUILD_FOLDER}/libnullpay_${TARGET_ARCH}/lib

    cp "${WORKDIR}/target/${TRIPLET}/release/libnullpay.a" ${ANDROID_BUILD_FOLDER}/libnullpay_${TARGET_ARCH}/lib
    cp "${WORKDIR}/target/${TRIPLET}/release/libnullpay.so" ${ANDROID_BUILD_FOLDER}/libnullpay_${TARGET_ARCH}/lib

     pushd ${WORKDIR}
        rm -f libnullpay_android_${TARGET_ARCH}.zip
        cp -rf ${ANDROID_BUILD_FOLDER}/libnullpay_${TARGET_ARCH} .
        if [ -z "${LIBNULLPAY_VERSION}" ]; then
            zip -r libnullpay_android_${TARGET_ARCH}.zip libnullpay_${TARGET_ARCH}
        else
            zip -r libnullpay_android_${TARGET_ARCH}_${LIBNULLPAY_VERSION}.zip libnullpay_${TARGET_ARCH}
        fi

    popd
}

build(){
    echo "**************************************************"
    echo "Building for architecture ${BOLD}${YELLOW}${TARGET_ARCH}${RESET}"
    echo "Toolchain path ${BOLD}${YELLOW}${TOOLCHAIN_DIR}${RESET}"
    echo "Sodium path ${BOLD}${YELLOW}${SODIUM_DIR}${RESET}"
    echo "Indy path ${BOLD}${YELLOW}${INDY_DIR}${RESET}"
    echo "Artifacts will be in ${BOLD}${YELLOW}${ANDROID_BUILD_FOLDER}/libindy_${TARGET_ARCH}${RESET}"
    echo "**************************************************"
    pushd ${WORKDIR}
        rm -rf target/${TRIPLET}
        cargo clean
        cargo build --release --target=${TRIPLET}
    popd
}


generate_arch_flags ${TARGET_ARCH}
setup_dependencies
download_and_unzip_dependencies_for_all_architectures
download_and_setup_toolchain
set_env_vars
create_standalone_toolchain_and_rust_target
create_cargo_config
build
package_library