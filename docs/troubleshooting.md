# Troubleshooting Guide

This guide contains solutions for common issues during development.

## Compilation

### Compilation Issues

If the firmware from the repository no longer compiles, make sure that you are using the correct Rust version.  Generally, we are using the latest stable Rust release.  If that does not work, you might want to use the stable Rust version at the time of the last commit (see the [Rust changelog][] for the release dates).

[Rust changelog]: https://github.com/rust-lang/rust/blob/master/RELEASES.md

## Debugging

### `arm-none-eabi-gdb` Not Found

`cargo run` per default uses the `arm-none-eabi-gdb` binary (see `runners/lpc55/.cargo/config`).  On some systems, this executable is called differently, for example `gdb-mulitarch` on Debian.  The easist persistent solution for this problem is to create a link with that name.

### NRF52 Debug Adapter Connection Issues

In case the NRF52 device was locked, the subsequent connections over debug adapter might not work.
To unlock it follow these:
- install `nrfjprog` (https://www.nordicsemi.com/Products/Development-tools/nrf-command-line-tools/download)
- make sure you have user access to the debug adapter (install udev rules)
- execute `make full-deploy` from `utils/nrf-builder`  using LPC-Link 2 (or nRF52840-DK) for a complete device recovery
    - alternative: execute `nrfjprog -f NRF52 --recover` using LPC-Link 2 (or nRF52840-DK - ST-Link will not work; a warning might show up, that connection has failed )
    - alternative: call `pyocd commander --target nrf52840 -O auto_unlock`, which tries to unlock the target as well
    - call mass erase to check if it succeeded

