FROM debian:jessie

RUN set -x

# Install dependencies

RUN apt-get update \
    && apt-get install -y curl \
                          git \
                          libgtk-3-dev \
                          libssl-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Compile libssl static lib

RUN OPENSSL_VERSION=1.0.2o \
    && curl -LO "https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz" \
	&& tar xvzf "openssl-$OPENSSL_VERSION.tar.gz" && cd "openssl-$OPENSSL_VERSION" \
    && ./config no-zlib -fPIC \
                --prefix=/usr/local  \
    && make \
    && make install

# Install rust as user rustacean

RUN useradd --create-home --shell /bin/bash rustacean
USER rustacean

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain stable -y

# Build as user rustacean

VOLUME /home/rustacean/src
WORKDIR /home/rustacean/src

COPY ./entrypoint.sh /home/rustacean/entrypoint.sh
CMD ["/home/rustacean/entrypoint.sh"]
