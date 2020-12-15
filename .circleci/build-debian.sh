#!/usr/bin/env bash

#
# Build for Debian in a docker container
#

# bailout on errors and echo commands.
set -xe

DOCKER_SOCK="unix:///var/run/docker.sock"

echo "DOCKER_OPTS=\"-H tcp://127.0.0.1:2375 -H $DOCKER_SOCK -s overlay2\"" | sudo tee /etc/default/docker > /dev/null
sudo service docker restart
sleep 5;

if [ "$EMU" = "on" ]; then
  if [ "$CONTAINER_DISTRO" = "raspbian" ]; then
      docker run --rm --privileged multiarch/qemu-user-static:register --reset
  else
      docker run --rm --privileged --cap-add=ALL --security-opt="seccomp=unconfined" multiarch/qemu-user-static --reset --credential yes --persistent yes
  fi
fi

WORK_DIR=$(pwd):/ci-source

docker run --privileged --cap-add=ALL --security-opt="seccomp=unconfined" -d -ti -e "container=docker"  -v $WORK_DIR:rw $DOCKER_IMAGE /bin/bash
DOCKER_CONTAINER_ID=$(docker ps --last 4 | grep $CONTAINER_DISTRO | awk '{print $1}')

docker exec --privileged -ti $DOCKER_CONTAINER_ID apt-get update
docker exec --privileged -ti $DOCKER_CONTAINER_ID apt-get -y install apt-transport-https wget curl gnupg2
docker exec --privileged -ti $DOCKER_CONTAINER_ID apt-get -y install dpkg-dev debhelper devscripts equivs pkg-config apt-utils fakeroot
docker exec --privileged -ti $DOCKER_CONTAINER_ID apt-get -y install cargo jekyll
docker exec --privileged -ti $DOCKER_CONTAINER_ID apt-get -y install libgtk-3-dev libssl-dev
docker exec --privileged -ti $DOCKER_CONTAINER_ID /bin/bash -xec \
    "update-alternatives --set fakeroot /usr/bin/fakeroot-tcp; cd ci-source; dpkg-buildpackage -b -uc -us -j4; mkdir dist; mv ../*.deb dist; chmod -R a+rw dist"

find dist -name \*.\*$EXT

echo "Stopping"
docker ps -a
docker stop $DOCKER_CONTAINER_ID
docker rm -v $DOCKER_CONTAINER_ID
