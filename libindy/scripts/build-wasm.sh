#!/usr/bin/env bash

WORKDIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
LIBINDYDIR=${WORKDIR}/..
BUILDDIR="/tmp/libindy"
if [[ ! -d "${BUILDDIR}" ]]; then
    mkdir ${BUILDDIR}
fi
DEPENDENCYDIR="${BUILDDIR}/deps"
if [[ ! -d "${DEPENDENCYDIR}" ]]; then
    mkdir ${DEPENDENCYDIR}
fi

LIBSODIUMPREBUILT="/tmp/libindy/deps/libsodium/libsodium-wasm32-wasi"

# install wapm
#curl https://get.wasmer.io -sSfL | sh

# install wasi
#curl https://raw.githubusercontent.com/wasienv/wasienv/master/install.sh | sh

source /Users/abdussami/.wasmer/wasmer.sh


# create wasm for sqlite
build-sqlite(){
    SQLITEDIR="${DEPENDENCYDIR}/sqlite"
    if [[ -d "${SQLITEDIR}" ]]; then
        rm -rf ${SQLITEDIR}
    fi
    mkdir ${SQLITEDIR}

    pushd "${SQLITEDIR}"
        wapm install sqlite
    popd
}



# create wasm for openssl
build-openssl(){
    OPENSSLDIR="${DEPENDENCYDIR}/openssl"
    if [[ -d "${OPENSSLDIR}" ]]; then
        rm -rf ${OPENSSLDIR}
    fi
    mkdir ${OPENSSLDIR}

    pushd "${OPENSSLDIR}"
#        wapm install openssl
        wget https://www.openssl.org/source/openssl-1.1.0h.tar.gz
        tar xf openssl-1.1.0h.tar.gz
        cd openssl-1.1.0h

        emconfigure ./Configure linux-generic64 --prefix=$EMSCRIPTEN/system

        sed -i 's|^CROSS_COMPILE.*$|CROSS_COMPILE=|g' Makefile

        emmake make -j 12 build_generated libssl.a libcrypto.a
        rm -rf $EMSCRIPTEN/system/include/openssl
        cp -R include/openssl $EMSCRIPTEN/system/include
        cp libcrypto.a libssl.a $EMSCRIPTEN/system/lib
        cd ..
        rm -rf openssl-1.1.0h*
    popd
}



# create wasm for libsodium
build-libsodium(){
    LIBSODIUMDIR="${DEPENDENCYDIR}/libsodium"
    if [[ -d "${LIBSODIUMDIR}" ]]; then
        rm -rf ${LIBSODIUMDIR}
    fi
    mkdir ${LIBSODIUMDIR}

    pushd "${LIBSODIUMDIR}"
        git clone https://github.com/jedisct1/libsodium.git
        pushd libsodium
#            git checkout 1.0.16
            env WASMER_DIR=${HOME}/.wasmer PATH=${HOME}/.wasmer/bin:/opt/wasi-sdk/bin:${HOME}/.cargo/bin:$PATH dist-build/wasm32-wasi.sh
         popd
    popd
}



# create wasm for libindy

build-libindy(){
    pushd "${LIBINDYDIR}"
        export PKG_CONFIG_ALLOW_CROSS=1
        export SODIUM_LIB_DIR="${LIBSODIUMPREBUILT}/lib"
        printenv
        cargo clean
        cargo build --target wasm32-unknown-unknown
    popd
}

#build-sqlite
#build-openssl
#build-libsodium
build-libindy