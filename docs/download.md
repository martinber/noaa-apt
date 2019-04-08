---
title: Download
layout: main
---

## Download

You can download everything from the
[releases page on GitHub](https://github.com/martinber/noaa-apt/releases).

The GUI version has an [easy to use interface](./usage.html#gui) but you can
[use it from the terminal too](./usage.html#terminal). The no-GUI version can be
[used only from the terminal](./usage.html#terminal).

### GNU/Linux 64 bit PC

#### Debian-based distros

- For Debian, Ubuntu, Linux Mint, etc. download the `.deb` package:

    [GNU/Linux x86_64 .deb package][amd64_deb].

    Install it by running `sudo apt install ~/Downloads/noaa-apt_0.9.8-1_amd64.deb`.

- If you are not going to use the GUI you can download the no-GUI executable from:

    [GNU/Linux x86_64 (no-GUI) zip][x86_64_linux_gnu_nogui_zip].

#### Other distros

- Download the executable with GUI from:

    [GNU/Linux x86_64 zip][x86_64_linux_gnu_zip].

- Download the executable without GUI from:

    [GNU/Linux x86_64 (no-GUI) zip][x86_64_linux_gnu_nogui_zip].

### Windows 64 bit PC

Right now you can download version 0.9.8 instead of 1.0.0.
[I'm having problems building the binaries](https://github.com/gtk-rs/gtk/issues/794),
sorry, I'm working on it.

- [Windows x86_64 zip][x86_64_windows_gnu_zip].

### Raspberry Pi 2+

- Download the executable with GUI from:

    [GNU/Linux armv7 zip][armv7_linux_gnueabihf_zip].

- Download the executable without GUI from:

    [GNU/Linux armv7 (no-GUI) zip][armv7_linux_gnueabihf_nogui_zip].

### OSX

- [Compile it yourself](./development.html#compilation).

### Something else?

- [Compile it yourself](./development.html#compilation).

## Dependencies

On Windows there aren't any dependencies, on Linux you probably already have
installed what you need:

- GTK+ >= 3.16 (Only for the GUI version)

- glibc >= 2.19

- libgcc

My builds use a statically linked libssl, so you don't need libssl unless you
compiled noaa-apt yourself.

[amd64_deb]: https://github.com/martinber/noaa-apt/releases/download/v1.0.0/noaa-apt_1.0.0-1_amd64.deb
[x86_64_windows_gnu_zip]: https://github.com/martinber/noaa-apt/releases/download/v0.9.8/noaa-apt-0.9.8-x86_64-windows-gnu.zip
[x86_64_linux_gnu_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.0.0/noaa-apt-1.0.0-x86_64-linux-gnu.zip
[x86_64_linux_gnu_nogui_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.0.0/noaa-apt-1.0.0-x86_64-linux-gnu-nogui.zip
[armv7_linux_gnueabihf_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.0.0/noaa-apt-1.0.0-armv7-linux-gnueabihf.zip
[armv7_linux_gnueabihf_nogui_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.0.0/noaa-apt-1.0.0-armv7-linux-gnueabihf-nogui.zip
