FROM debian:stretch

RUN set -x

# Install dependencies

RUN apt-get update \
    && apt-get install -y curl \
                          git \
                          zip \
                          libgtk-3-dev \
                          debhelper \
                          gcc-6-arm-linux-gnueabihf \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Compile libssl static lib for x86 and armv7

RUN OPENSSL_VERSION=1.0.2o \
    && curl -LO "https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz" \
    && tar xvzf "openssl-$OPENSSL_VERSION.tar.gz" && cd "openssl-$OPENSSL_VERSION" \
    && ./config no-zlib -fPIC \
                --prefix=/usr/local/openssl \
    && make \
    && make install \
    && make clean \
    && export MACHINE=armv7 ARC=arm CC=arm-linux-gnueabihf-gcc-6 \
    && ./config shared \
                --prefix=/usr/local/openssl_armv7 \
    && make  \
    && make install

# Install rust as user rustacean

RUN useradd --create-home --shell /bin/bash rustacean
USER rustacean

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain stable -y \
    && /home/rustacean/.cargo/bin/rustup target add armv7-unknown-linux-gnueabihf

ADD .cargo/config /home/rustacean/.cargo/config

# Build as user rustacean

VOLUME /home/rustacean/src
WORKDIR /home/rustacean/src

COPY ./entrypoint.sh /home/rustacean/entrypoint.sh
CMD ["/home/rustacean/entrypoint.sh"]
