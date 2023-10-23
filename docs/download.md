---
title: Download
layout: main
---

## Download

The relevant downloads from the
[releases page on GitHub](https://github.com/martinber/noaa-apt/releases) are
listed below. You can also download/clone the GitHub repository, but I recommend
using one of the links below.

The GUI version is recommended because it includes
[an easy to use graphical interface](./usage.html#gui), and also can be
[used from the terminal](./usage.html#terminal). The no-GUI
version can be [used only from the terminal](./usage.html#terminal).

After downloading and installing, visit [the Usage page](./usage.html).

### GNU/Linux 64 bit PC

#### Debian-based distros (Ubuntu, Linux Mint, etc.)

- `.deb` package with GUI (recommended):

    [GNU/Linux x86_64 .deb package][amd64_deb].

    You can install it on some distros by double clicking the file and clicking
    an "Install" button. Otherwise open a terminal and install it by running:

    ```
    sudo apt install ~/Downloads/noaa-apt_1.4.0-1_amd64.deb
    ```

    To run, search for noaa-apt in your installed programs or run `noaa-apt` in
    a terminal

- Executable without GUI (for a terminal):

    [GNU/Linux x86_64 (no-GUI) zip][x86_64_linux_gnu_nogui_zip].

    To install, extract the zip file anywhere.

#### Arch Linux and similar distros

- Sylogista maintains an [AUR package](https://aur.archlinux.org/packages/noaa-apt/)

#### NixOS

- Tom Repetti maintains a [Nix package](https://search.nixos.org/packages?query=noaa-apt&from=0&size=30&sort=relevance&channel=unstable)

#### Other distros

- Executable with GUI (recommended):

    [GNU/Linux x86_64 zip][x86_64_linux_gnu_zip].

    To install, extract the zip file anywhere. To run, double click
    `run-noaa-apt.sh`.

- Executable without GUI (for a terminal):

    [GNU/Linux x86_64 (no-GUI) zip][x86_64_linux_gnu_nogui_zip].

    To install, extract the zip file anywhere.

### Windows 64 bit PC

- [Windows x86_64 zip][x86_64_windows_gnu_zip].

    To install, extract the zip file anywhere.

### Raspberry Pi 2/3 (armv7, armhf)

- Executable with GUI (recommended):

    [GNU/Linux armv7 zip][armv7_linux_gnueabihf_zip].

    To install, extract the zip file anywhere. To run, double click
    `run-noaa-apt.sh`.

- Executable without GUI (for a terminal):

    [GNU/Linux armv7 (no-GUI) zip][armv7_linux_gnueabihf_nogui_zip].

    To install, extract the zip file anywhere.

### Raspberry Pi 4/5 (armv8, aarch64, arm64)

- Executable with GUI (recommended):

    [GNU/Linux aarch64 zip][aarch64_linux_gnu_zip].

    To install, extract the zip file anywhere. To run, double click
    `run-noaa-apt.sh`.

- Executable without GUI (for a terminal):

    [GNU/Linux aarch64 (no-GUI) zip][aarch64_linux_gnu_nogui_zip].

    To install, extract the zip file anywhere.

### OSX

- You can use the [Nix Package](https://search.nixos.org/packages?query=noaa-apt&from=0&size=30&sort=relevance&channel=unstable).
    First, install Nix, then install the `noaa-apt` package and then run the
    program from the terminal:

    ```
    sh <(curl -L https://nixos.org/nix/install)
    nix-env -iA nixos.noaa-apt
    noaa-apt
    ```

- [Otherwise compile it yourself following these instructions](./development.html#compilation).

### Android+Termux

- [Compile it yourself following these instructions](./development.html#compilation).

### Something else?

- [Compile it yourself following these instructions](./development.html#compilation).

## Dependencies

On Windows there aren't any dependencies, on Linux you probably already have
installed what you need:

- libc6 (>= 2.28)
- libcairo-gobject2 (>= 1.10.0)
- libcairo2 (>= 1.2.4)
- libgcc1 (>= 1:4.2)
- libgdk-pixbuf2.0-0 (>= 2.31.1)
- libglib2.0-0 (>= 2.31.8)
- libgtk-3-0 (>= 3.21.4)
- libpango-1.0-0 (>= 1.14.0)

My builds use a statically linked libssl, so you don't need libssl unless you
compiled noaa-apt yourself.

[amd64_deb]: https://github.com/martinber/noaa-apt/releases/download/v1.4.0/noaa-apt_1.4.0-1_amd64.deb
[x86_64_windows_gnu_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.4.0/noaa-apt-1.4.0-x86_64-windows-gnu.zip
[x86_64_linux_gnu_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.4.0/noaa-apt-1.4.0-x86_64-linux-gnu.zip
[x86_64_linux_gnu_nogui_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.4.0/noaa-apt-1.4.0-x86_64-linux-gnu-nogui.zip
[armv7_linux_gnueabihf_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.4.0/noaa-apt-1.4.0-armv7-linux-gnueabihf.zip
[armv7_linux_gnueabihf_nogui_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.4.0/noaa-apt-1.4.0-armv7-linux-gnueabihf-nogui.zip
[aarch64_linux_gnu_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.4.0/noaa-apt-1.4.0-aarch64-linux-gnu.zip
[aarch64_linux_gnu_nogui_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.4.0/noaa-apt-1.4.0-aarch64-linux-gnu-nogui.zip
