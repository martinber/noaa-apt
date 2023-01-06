FROM debian:buster

RUN set -x

# I install GTK+3 libs with the armhf and aarch64 architectures one by one.
# I think I cannot have all of them at the same time, so I will end up only with
# the x86 GTK, but at least all the dependencies for the other architectures
# will remain

RUN dpkg --add-architecture armhf \
    && dpkg --add-architecture arm64 \
    && apt-get update \
    && apt-get install -y curl \
                          git \
                          zip \
                          gcc-8-arm-linux-gnueabihf \
                          gcc-8-aarch64-linux-gnu \
                          debhelper \
    && apt-get install -y libgtk-3-dev:armhf \
    && apt-get install -y libgtk-3-dev:arm64 \
    && apt-get install -y libgtk-3-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install rust as user rustacean

RUN useradd --create-home --shell /bin/bash rustacean
USER rustacean

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain stable -y \
    && /home/rustacean/.cargo/bin/rustup target add armv7-unknown-linux-gnueabihf \
    && /home/rustacean/.cargo/bin/rustup target add aarch64-unknown-linux-gnu

VOLUME /home/rustacean/src
WORKDIR /home/rustacean/src

COPY .cargo/config /home/rustacean/.cargo/config
COPY ./entrypoint.sh /home/rustacean/entrypoint.sh
ENTRYPOINT ["/home/rustacean/entrypoint.sh"]
