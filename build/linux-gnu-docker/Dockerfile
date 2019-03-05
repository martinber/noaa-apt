FROM debian:stretch

RUN set -x

# Install basic dependencies and install GTK+3 libs with the armhf architecture
# Them copy the GTK+3 libs I need to a temporary folder, install the libs for
# x86_64 and then copy the libs I wanted to the location I need.

# So I end up with the GTK+3 libs for x86_64 but also with some files for armhf.
# I do this because I can't install both at the same time.

RUN dpkg --add-architecture armhf \
    && apt-get update \
    && apt-get install -y curl \
                          git \
                          zip \
                          gcc-6-arm-linux-gnueabihf \
                          debhelper \
                          libgtk-3-dev:armhf \
    && mkdir -p /tmp/gtklibs/usr/lib/ \
    && mkdir -p /tmp/gtklibs/lib/ \
    && cp -r /usr/lib/arm-linux-gnueabihf/ /tmp/gtklibs/usr/lib/ \
    && cp -r /lib/arm-linux-gnueabihf/ /tmp/gtklibs/lib/ \
    && apt-get install -y libgtk-3-dev \
    && cp -r /tmp/gtklibs/usr/lib/arm-linux-gnueabihf/ /usr/lib/ \
    && cp -r /tmp/gtklibs/lib/arm-linux-gnueabihf/ /lib/ \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && rm -r /tmp/*

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
