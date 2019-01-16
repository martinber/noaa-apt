---
title: Download
layout: main
---

## Download

There is a GUI version and a no-GUI version available.

- [noaa-apt 0.9.7 GNU/Linux x86_64](https://github.com/martinber/noaa-apt/releases/download/v0.9.7/noaa-apt-0.9.7-x86_64-linux-gnu.zip).

- [noaa-apt 0.9.7 GNU/Linux x86_64 (no GUI)](https://github.com/martinber/noaa-apt/releases/download/v0.9.7/noaa-apt-0.9.7-x86_64-linux-gnu-nogui.zip).

- [noaa-apt 0.9.7 Windows x86_64](https://github.com/martinber/noaa-apt/releases/download/v0.9.7/noaa-apt-0.9.7-x86_64-windows-gnu.zip).

You can download those executables, or old ones from the
[releases page on GitHub](https://github.com/martinber/noaa-apt/releases).

Your options are:

- GNU/Linux:

    - Download executable with GUI, (needs GTK and GLIBC version at least
      2.19).

    - Download executable without GUI.

    - [Compile it yourself](./development.html#compilation).

- Windows:

    - Download executable with GUI.

    - [Compile the GUI version yourself from GNU/Linux](./development.html#compilation).

    - Compile the no-gui version, or compile from Windows but I don't know how to
      do it.

- OSX:

  - [Compile it yourself](./development.html#compilation).

- Something else?

    - At least the no-gui version should work everywhere (Raspberry Pi?) because
      there are no dependencies apart from GTK and a handful of pure Rust
      modules.
      [You should compile it yourself though](./development.html#compilation).

## Dependencies

On Windows there aren't any dependencies, on Linux you probably already have
installed what you need:

- GTK+ > 3.16

- TODO
