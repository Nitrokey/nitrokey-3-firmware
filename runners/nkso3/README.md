# STM32N6 test firmware

This directory contains a minimal test firmware for the STM32N6.

## Requirements

- NUCLEO-N657X0-Q development board
- Rust as specified in [`rust-toolchain.toml`](../../rust-toolchain.toml)
- pyOCD
- `arm-none-eabi-gdb`

## Getting Started

- Install the required CMSIS pack for pyOCD with `pyocd pack install STM32N657X0HxQ`.
- Make sure you have the necessary udev rules for the STLINK-V3 (0x0483:0x3754), see for example [the pyOCD udev rules](https://github.com/pyocd/pyOCD/tree/main/udev).
- Check that the JP2 jumper (BOOT1) on the development board is set to 2-3 (BOOT1 = 1) to enable the development boot mode.
- Connect the development board via the CN10 USB connector and check that it appears in `lsusb` and `pyocd list`.
- Start the GDB server with `make gdbserver`.
- Compile and flash the test firmware with `make run`.
- Observe that the three LEDs LD5, LD6 and LD7 turn on and that you see the semihosting messages in the output of the GDB server.
