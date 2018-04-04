FROM japaric/armv7-linux-androideabi:v0.1.14
RUN apt-get update && \
    apt-get install -y curl \
    unzip

COPY prepare_binaries.sh /
RUN bash /prepare_binaries.sh arm
ENV SODIUM_LIB_DIR=/tmp/prebuilt_deps/android/deps/armv7/libsodium_1.0.16/lib \
    LIBZMQ_PREFIX=/tmp/prebuilt_deps/android/deps/armv7/libzmq_4.2.4 \
    PKG_CONFIG_ALLOW_CROSS=1 \
    CARGO_INCREMENTAL=1 \
    RUST_LOG=indy=trace \
    RUST_TEST_THREADS=1 \
    RUST_BACKTRACE=1