#!/usr/bin/env bash

# Find the newest version of glibc required by a binary, and compare it to a
# fixed, maximum allowed version

# Minor modifications to script by Alysha Gardner:
# https://www.agardner.me/golang/cgo/c/dependencies/glibc/kernel/linux/2015/12/12/c-dependencies.html

BIN="$1"
MAX_ALLOWED_VER="2.19"

if [ -z "$BIN" ] || [ -z "$MAX_ALLOWED_VER" ]; then
  echo "Usage: glibc_check.sh <binary>"
  exit 1
fi

MAX_ALLOWED_MAJ_VER="$(echo $MAX_ALLOWED_VER | cut -f 1 -d '.')"
MAX_ALLOWED_MIN_VER="$(echo $MAX_ALLOWED_VER | cut -f 2 -d '.')"

# Get the max major version, then find the max minor version for that major
MAX_MAJ_VER="$(objdump -T "$BIN" | sed -n 's/.*GLIBC_\([0-9]\.[0-9]\+\).*$/\1/p' | cut -f 1 -d '.' | sort -g | tail -n 1)"
MAX_MIN_VER="$(objdump -T "$BIN" | grep GLIBC_$MAX_MAJ_VER | sed -n 's/.*GLIBC_\([0-9]\.[0-9]\+\).*$/\1/p' | cut -f 2 -d '.' | sort -g | tail -n 1)"
MAX_VER="$MAX_MAJ_VER.$MAX_MIN_VER"

if echo "$MAX_MAJ_VER $MAX_ALLOWED_MAJ_VER $MAX_MIN_VER $MAX_ALLOWED_MIN_VER" | awk '{exit $1>$2||$3>$4?0:1}'; then
	echo "FAIL - Got max GLIBC version $MAX_VER, greater than allowed max $MAX_ALLOWED_VER"
	exit 1
else
	echo "OK - Got max GLIBC version $MAX_VER, less than or equal to allowed max $MAX_ALLOWED_VER"
fi
