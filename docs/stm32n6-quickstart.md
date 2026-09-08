# STM32N6 quickstart

Board setup for running the `nkso3` firmware on the NUCLEO-N657X0-Q.
Building and running is done with [`utils/stm32n6-builder`](../utils/stm32n6-builder/README.md).

## Requirements

- NUCLEO-N657X0-Q development board
- Rust as specified in [`rust-toolchain.toml`](../rust-toolchain.toml)
- pyOCD
- `arm-none-eabi-gdb`

## Board setup

- Install the CMSIS pack for pyOCD with `pyocd pack install STM32N657X0HxQ`.
- Add udev rules for the STLINK-V3 (0x0483:0x3754), see for example
  [the pyOCD udev rules](https://github.com/pyocd/pyOCD/tree/main/udev).
- Set the JP2 jumper (BOOT1) to 2-3 (BOOT1 = 1) to enable the development boot mode.
  The firmware is loaded into RAM over the ST-LINK and is lost on power cycle.
- Connect the ST-LINK USB connector (CN10) and check that the board appears in `lsusb`
  and `pyocd list`.
- Connect the user USB-C connector (CN8) to the host; this is where the firmware
  enumerates as a Nitrokey device.

## Running

```
$ make -C utils/stm32n6-builder gdbserver
$ make -C utils/stm32n6-builder run FEATURES=develop,log-semihosting
```

The semihosting log appears in the GDB server output, and the device shows up as
`20a0:42b2` in `lsusb`.
