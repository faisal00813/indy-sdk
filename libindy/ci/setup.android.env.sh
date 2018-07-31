#!/usr/bin/env bash


if [ -z "${ANDROID_BUILD_FOLDER}" ]; then
    echo STDERR "ANDROID_BUILD_FOLDER is not set. Please set it in the caller script"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi
ANDROID_SDK=${ANDROID_BUILD_FOLDER}/sdk
export ANDROID_SDK_ROOT=${ANDROID_SDK}
export ANDROID_HOME=${ANDROID_SDK}
export PATH=${PATH}:${ANDROID_HOME}/platform-tools
export PATH=${PATH}:${ANDROID_HOME}/tools
export PATH=${PATH}:${ANDROID_HOME}/tools/bin

mkdir -p ${ANDROID_SDK}

TARGET_ARCH=$1

if [ -z "${TARGET_ARCH}" ]; then
    echo STDERR "Missing TARGET_ARCH argument"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi



check_if_emulator_is_running(){
    emus=$(adb devices)
    if [[ ${emus} = *"emulator"* ]]; then
      echo "emulator is running"
      else
       echo "emulator is not running"
       exit 1
    fi
}

kill_avd(){
    adb devices | grep emulator | cut -f1 | while read line; do adb -s $line emu kill; done
}
delete_existing_avd(){
    kill_avd
    avdmanager delete avd -n ${ARCH}
}

create_avd(){

    echo "${GREEN}Creating Android SDK${RESET}"
    echo "yes" | \
          sdkmanager --no_https \
            "emulator" \
            "platform-tools" \
            "platforms;android-24" \
            "system-images;android-24;default;${ABI}"

    echo "${BLUE}Creating android emulator${RESET}"

        echo "no" |
             avdmanager create avd \
                --name ${TARGET_ARCH} \
                --package "system-images;android-24;default;${ABI}" \
                -f \
                -c 1000M

        ANDROID_SDK_ROOT=${ANDROID_SDK} ANDROID_HOME=${ANDROID_SDK} ${ANDROID_HOME}/tools/emulator -avd ${TARGET_ARCH}
        ANDROID_SDK_ROOT=${ANDROID_SDK} ANDROID_HOME=${ANDROID_SDK} ${ANDROID_HOME}/tools/emulator -avd arm -no-audio -no-window &
}

download_sdk(){
    echo "${GREEN}Downloading sdk....${RESET}"
     pushd ${ANDROID_SDK}
        wget https://dl.google.com/android/repository/sdk-tools-linux-4333796.zip
        echo "${GREEN}Done!${RESET}"
        unzip -qq sdk-tools-linux-4333796.zip
        delete_existing_avd
        create_avd
     popd
}


generate_arch_flags(){
    if [ -z $1 ]; then
        echo STDERR "${RED}Please provide the arch e.g arm,armv7, x86 or arm64${RESET}"
        exit 1
    fi
    if [ $1 == "arm" ]; then
        export TARGET_ARCH="arm"
        export TARGET_API="16"
        export TRIPLET="arm-linux-androideabi"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="armeabi-v7a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "armv7" ]; then
        export TARGET_ARCH="arm"
        export TARGET_API="16"
        export TRIPLET="armv7-linux-androideabi"
        export ANDROID_TRIPLET="arm-linux-androideabi"
        export ABI="armeabi-v7a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "arm64" ]; then
        export TARGET_ARCH="arm64"
        export TARGET_API="21"
        export TRIPLET="aarch64-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="arm64-v8a"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "x86" ]; then
        export TARGET_ARCH="x86"
        export TARGET_API="16"
        export TRIPLET="i686-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="x86"
        export TOOLCHAIN_SYSROOT_LIB="lib"
    fi

    if [ $1 == "x86_64" ]; then
        export TARGET_ARCH="x86_64"
        export TARGET_API="21"
        export TRIPLET="x86_64-linux-android"
        export ANDROID_TRIPLET=${TRIPLET}
        export ABI="x86_64"
        export TOOLCHAIN_SYSROOT_LIB="lib64"
    fi

}

download_and_unzip_dependencies_for_all_architectures(){
    #TODO Get dependencies in more optimized way
    pushd ${ANDROID_BUILD_FOLDER}
        if [ ! -d "indy-android-dependencies" ] ; then
            echo "${GREEN}Downloading dependencies...${RESET}"
            git clone https://github.com/evernym/indy-android-dependencies.git
            pushd ${ANDROID_BUILD_FOLDER}/indy-android-dependencies/prebuilt/
                git checkout tags/v1.0.2
                find . -name "*.zip" | xargs -P 5 -I FILENAME sh -c 'unzip -o -qq -d "$(dirname "FILENAME")" "FILENAME"'
            popd
             echo "${GREEN}Done!${RESET}"
        fi
        export OPENSSL_DIR=${ANDROID_BUILD_FOLDER}/indy-android-dependencies/prebuilt/openssl/openssl_${TARGET_ARCH}
        export SODIUM_DIR=${ANDROID_BUILD_FOLDER}/indy-android-dependencies/prebuilt/sodium/libsodium_${TARGET_ARCH}
        export LIBZMQ_DIR=${ANDROID_BUILD_FOLDER}/indy-android-dependencies/prebuilt/zmq/libzmq_${TARGET_ARCH}
	popd
}



create_standalone_toolchain_and_rust_target(){
    #will only create toolchain if not already created
    python3 ${ANDROID_NDK_ROOT}/build/tools/make_standalone_toolchain.py \
    --arch ${TARGET_ARCH} \
    --api ${TARGET_API} \
    --stl=gnustl \
    --install-dir ${TOOLCHAIN_DIR}

    # add rust target
    rustup target add ${TRIPLET}
}



download_and_setup_toolchain(){
    if [ "$(uname)" == "Darwin" ]; then
        export TOOLCHAIN_PREFIX=${ANDROID_BUILD_FOLDER}/toolchains/darwin
        mkdir -p ${TOOLCHAIN_PREFIX}
        pushd $TOOLCHAIN_PREFIX
        if [ ! -d "android-ndk-r16b" ] ; then
            echo "${GREEN}Downloading NDK for OSX${RESET}"
            echo "${BLUE}Downloading... android-ndk-r16b-darwin-x86_64.zip${RESET}"
            wget -q https://dl.google.com/android/repository/android-ndk-r16b-darwin-x86_64.zip
            unzip -qq android-ndk-r16b-darwin-x86_64.zip
            echo "${GREEN}Done!${RESET}"
        else
            echo "${BLUE}Skipping download android-ndk-r16b-darwin-x86_64.zip${RESET}"
        fi
        export ANDROID_NDK_ROOT=${TOOLCHAIN_PREFIX}/android-ndk-r16b
        popd
    elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
        export TOOLCHAIN_PREFIX=${ANDROID_BUILD_FOLDER}/toolchains/linux
        mkdir -p ${TOOLCHAIN_PREFIX}
        pushd $TOOLCHAIN_PREFIX
        if [ ! -d "android-ndk-r16b" ] ; then
            echo "${GREEN}Downloading NDK for Linux${RESET}"
            echo "${BLUE}Downloading... android-ndk-r16b-linux-x86_64.zip${RESET}"
            wget -q https://dl.google.com/android/repository/android-ndk-r16b-linux-x86_64.zip
            unzip -qq android-ndk-r16b-linux-x86_64.zip
            echo "${GREEN}Done!${RESET}"
        else
            echo "${BLUE}Skipping download android-ndk-r16b-linux-x86_64.zip${RESET}"
        fi
        export ANDROID_NDK_ROOT=${TOOLCHAIN_PREFIX}/android-ndk-r16b
        popd
    fi

}


set_env_vars(){
    export PKG_CONFIG_ALLOW_CROSS=1
    export CARGO_INCREMENTAL=1
    export RUST_LOG=indy=trace
    export RUST_TEST_THREADS=1
    export RUST_BACKTRACE=1
    export OPENSSL_DIR=${OPENSSL_DIR}
    export SODIUM_LIB_DIR=${SODIUM_DIR}/lib
    export SODIUM_INCLUDE_DIR=${SODIUM_DIR}/include
    export LIBZMQ_LIB_DIR=${LIBZMQ_DIR}/lib
    export LIBZMQ_INCLUDE_DIR=${LIBZMQ_DIR}/include
    export TOOLCHAIN_DIR=${TOOLCHAIN_PREFIX}/${TARGET_ARCH}
    export PATH=${TOOLCHAIN_DIR}/bin:${PATH}
    export PKG_CONFIG_ALLOW_CROSS=1
    export CC=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-clang
    export AR=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ar
    export CXX=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-clang++
    export CXXLD=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ld
    export RANLIB=${TOOLCHAIN_DIR}/bin/${ANDROID_TRIPLET}-ranlib
    export TARGET=android
    export OPENSSL_STATIC=1
}
