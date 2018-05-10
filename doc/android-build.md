# Building binaries of LibIndy for Android
Build process(cross-compilation) of libindy depends on the Host and Target

Host = The system on which build process will takes place (like osx or linux)

Target = The system on which the build binary will actually work. (like android or ios)

(You have to know the cpu architecture of target system)

## Setting up Toolchains
### Via Script
- make sure you have rust installed via rustup
- run 
    ```
    sudo apt-get update && \
    sudo apt-get install -y wget \
        unzip \
        curl \
        python \
        gcc \
        pkg-config
    ```
- run `sh install_toolchains.sh`

### Manual
This is a one time step needed to setup toolchains
- Install rust 
- run `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android` to install targets for rust
- create a dir called `NDK_TOOLCHAINS` wherever you like.
- cd into  `NDK_TOOLCHAINS` folder 
- run `export NDK_TOOLCHAIN_DIR=${PWD}`
- Download [Android NDK](https://dl.google.com/android/repository/android-ndk-r16b-darwin-x86_64.zip)
    - If your host is Linux download the respective NDK from [here](https://developer.android.com/ndk/downloads/index.html)
- Extract zip file
- cd into extracted directory
- run `export NDK_HOME=${PWD}`
- run 
```
${NDK_HOME}/build/tools/make_standalone_toolchain.py  --api 21 --arch arm64 --install-dir ${NDK_TOOLCHAIN_DIR}/arm64
${NDK_HOME}/build/tools/make_standalone_toolchain.py  --api 14 --arch arm --install-dir ${NDK_TOOLCHAIN_DIR}/arm
${NDK_HOME}/build/tools/make_standalone_toolchain.py  --api 14 --arch x86 --install-dir ${NDK_TOOLCHAIN_DIR}/x86
```
- goto ~/.cargo folder
- create a file called `config` with the following contents
```
[target.aarch64-linux-android]
ar = "<NDK_TOOLCHAINS folder>/arm64/bin/aarch64-linux-android-ar"
linker = "<NDK_TOOLCHAINS folder>/arm64/bin/aarch64-linux-android-clang"

[target.armv7-linux-androideabi]
ar = "<NDK_TOOLCHAINS folder>/arm/bin/arm-linux-androideabi-ar"
linker = "<NDK_TOOLCHAINS folder>/arm/bin/arm-linux-androideabi-clang"

[target.i686-linux-android]
ar = "<NDK_TOOLCHAINS folder>/x86/bin/i686-linux-android-ar"
linker = "<NDK_TOOLCHAINS folder>/x86/bin/i686-linux-android-clang"
```

## Build LibIndy
### Host: OSX 
### Target: android-linux-aarm64
- Follow the instructions except 4 & 5 [here](https://github.com/hyperledger/indy-sdk/blob/master/doc/mac-build.md) to install all the dependencies for indy-sdk 
- run `brew install zeromq` to install zeromq on osx
- Make sure you have setup the toolchains for OSX host
- run `sh android_build.sh aarm64`

### Host: Linux 
### Target: android-linux-armv7

- run 
```
sudo apt-get update && \
sudo apt-get install -y wget \
    unzip \
    curl \
    python \
    gcc \
    pkg-config

```
- Make sure you have setup the toolchains for Linux host
- run `sh android-build.sh armv7`

## Notes:
Make sure the Android app which is going to use lib-indy has permissions to write to external storage. 

Add following line to AndroidManifest.xml

`<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE"/>`


