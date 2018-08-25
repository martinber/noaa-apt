FROM debian:jessie

RUN set -x

RUN apt-get update && apt-get install -y curl libgtk-3-dev

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain stable -y

VOLUME /src
WORKDIR /src
ENV PKG_CONFIG_ALLOW_CROSS=1

COPY ./entrypoint.sh /entrypoint.sh
CMD ["/docker_entrypoint.sh"]
