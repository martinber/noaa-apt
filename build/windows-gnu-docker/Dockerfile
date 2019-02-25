# Use fedora or Arch because mingw64-gtk3 is not available on Debian.
FROM fedora:27

RUN set -x

# Install dependencies

RUN dnf -y update \
    && dnf clean all \
    && dnf -y install file gcc make man sudo tar zip \
                      mingw64-gcc \
                      mingw64-freetype freetype freetype-devel \
                      mingw64-cairo mingw64-cairo-static cairo cairo-devel \
                      mingw64-harfbuzz harfbuzz harfbuzz-devel \
                      mingw64-pango pango pango-devel \
                      mingw64-poppler poppler poppler-devel \
                      mingw64-gtk3 gtk3 gtk3-devel \
                      mingw64-glib2-static glib2 glib2-devel \
                      atk atk-devel \
                      mingw64-winpthreads mingw64-winpthreads-static \
    && dnf clean all

# Install rust as user rustacean

RUN useradd --create-home --shell /bin/bash rustacean
USER rustacean

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain stable -y

RUN /home/rustacean/.cargo/bin/rustup target add x86_64-pc-windows-gnu
ADD .cargo/config /home/rustacean/.cargo/config

# Build as user rustacean

VOLUME /home/rustacean/src
WORKDIR /home/rustacean/src
ENV PKG_CONFIG_ALLOW_CROSS=1

COPY ./entrypoint.sh /home/rustacean/entrypoint.sh
CMD ["/home/rustacean/entrypoint.sh"]
