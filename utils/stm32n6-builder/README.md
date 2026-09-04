# stm32n6 builder

This directory contains a `Makefile` for building and running the `stm32n6` firmware for the NUCLEO-N657X0-Q board using the embedded runner in [`../../runners/embedded`](../../runners/embedded).
See [`docs/stm32n6-quickstart.md`](../../docs/stm32n6-quickstart.md) for the board setup.

## Building the firmware

```
$ make build
```

## Running the firmware interactively

The firmware is loaded into RAM over the on-board ST-LINK and started with gdb.
First start the pyOCD GDB server (with semihosting enabled):

```
$ make gdbserver
```

Then build, load and run the firmware and enter the debugger:

```
$ make run FEATURES=develop,log-semihosting
```

The `size` target prints the section sizes of the last build.
