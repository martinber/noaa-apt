FROM fedora:27

########## SYSTEM SETUP ##########
RUN dnf -y update; dnf clean all
# fundamental packages 
RUN dnf -y install file gcc make man sudo tar; dnf clean all
# Download Rustup installer
RUN curl https://sh.rustup.rs -o /usr/bin/rustup-install
RUN chmod +x /usr/bin/rustup-install

########## RUST ##########
RUN useradd -ms /bin/bash rustacean
USER rustacean
# Install Rustup
RUN rustup-install -y
# Install Rust
RUN /home/rustacean/.cargo/bin/rustup update
RUN /home/rustacean/.cargo/bin/rustup target add x86_64-pc-windows-gnu
# Install target config
ADD .cargo/config /home/rustacean/.cargo/config

########## MINGW & BUILD DEPENDENCIES ##########
USER root
RUN dnf -y install mingw64-gcc
RUN dnf -y install mingw64-freetype freetype freetype-devel
RUN dnf -y install mingw64-cairo mingw64-cairo-static cairo cairo-devel
RUN dnf -y install mingw64-harfbuzz harfbuzz harfbuzz-devel
RUN dnf -y install mingw64-pango pango pango-devel
RUN dnf -y install mingw64-poppler poppler poppler-devel
RUN dnf -y install mingw64-gtk3 gtk3 gtk3-devel
RUN dnf -y install mingw64-glib2-static glib2 glib2-devel
RUN dnf -y install atk atk-devel 
RUN dnf -y install mingw64-winpthreads mingw64-winpthreads-static



########## CONFIG ##########
USER rustacean
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH=/usr/i686-w64-mingw32/lib/pkgconfig
# Mount your project here and docker start
VOLUME /home/rustacean/src
WORKDIR /home/rustacean/src


COPY ./docker_entrypoint.sh /home/rustacean/docker_entrypoint.sh
# RUN chmod +x /home/rustacean/docker_entrypoint.sh
# CMD ["/home/rustacean/.cargo/bin/cargo", "build", "--target=x86_64-pc-windows-gnu", "--release"]
CMD ["/home/rustacean/docker_entrypoint.sh"]

# So one could build a project in the current directory, where this Dockerfile is by
#   1) Modifying the Dockerfile to add all your native dependencies
#   2) Building the image:
#       $ docker build . -t PROJECTNAME-build-image
#   3) Creating a container with the source mounted the image (which kicks off the build):
#       $ docker create -v `pwd`:/home/rustacean/src --name PROJECTNAME-build PROJECTNAME-build-image
#   4) Each time you want to build the project, start the Docker container. 
#      Add "-ai" to watch the build progress.
#       $ docker start PROJECTNAME-build
