---
title: Download
layout: main
---

## Download

The relevant downloads from the
[releases page on GitHub](https://github.com/martinber/noaa-apt/releases) are
listed below. You can also download/clone the GitHub repository, but I recommend
using one of the links below.

The GUI version has [an easy to use graphical interface](./usage.html#gui) but
you can [use it from the terminal too](./usage.html#terminal). The no-GUI
version can be [used only from the terminal](./usage.html#terminal).

After downloading and installing, visit [the Usage page](./usage.html).

### GNU/Linux 64 bit PC

#### Debian-based distros (Ubuntu, Linux Mint, etc.)

- GUI version `.deb` package:

    [GNU/Linux x86_64 .deb package][amd64_deb].

    You can install it on some distros by double clicking the file and clicking
    an "Install" button. Otherwise open a terminal and install it by running:

    ```
    sudo apt install ~/Downloads/noaa-apt_X.X.X-1_amd64.deb
    ```

- Executable without GUI:

    [GNU/Linux x86_64 (no-GUI) zip][x86_64_linux_gnu_nogui_zip].

    To install, extract the zip file anywhere.

#### Arch Linux and similar distros

- Sylogista maintains an [AUR package](https://aur.archlinux.org/packages/noaa-apt/)

#### Other distros

- Executable with GUI:

    [GNU/Linux x86_64 zip][x86_64_linux_gnu_zip].

    To install, extract the zip file anywhere.

- Executable without GUI:

    [GNU/Linux x86_64 (no-GUI) zip][x86_64_linux_gnu_nogui_zip].

    To install, extract the zip file anywhere.

### Windows 64 bit PC

- [Windows x86_64 zip][x86_64_windows_gnu_zip].

    To install, extract the zip file anywhere.

### Raspberry Pi 2+

- Executable with GUI:

    [GNU/Linux armv7 zip][armv7_linux_gnueabihf_zip].

    To install, extract the zip file anywhere.

- Executable without GUI:

    [GNU/Linux armv7 (no-GUI) zip][armv7_linux_gnueabihf_nogui_zip].

    To install, extract the zip file anywhere.

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

[amd64_deb]: https://github.com/martinber/noaa-apt/releases/download/v1.1.1/noaa-apt_1.1.1-1_amd64.deb
[x86_64_windows_gnu_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.1.1/noaa-apt-1.1.1-x86_64-windows-gnu.zip
[x86_64_linux_gnu_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.1.1/noaa-apt-1.1.1-x86_64-linux-gnu.zip
[x86_64_linux_gnu_nogui_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.1.1/noaa-apt-1.1.1-x86_64-linux-gnu-nogui.zip
[armv7_linux_gnueabihf_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.1.1/noaa-apt-1.1.1-armv7-linux-gnueabihf.zip
[armv7_linux_gnueabihf_nogui_zip]: https://github.com/martinber/noaa-apt/releases/download/v1.1.1/noaa-apt-1.1.1-armv7-linux-gnueabihf-nogui.zip
