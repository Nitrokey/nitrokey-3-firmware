# lpc55 builder

This directory contains a `Makefile` for building the `lpc55` firmware using the embedded runner in [`../../runners/embedded`](../../runners/embedded).

## Building the firmware

```
$ make build
```

## Running the firmware interactively

If you have a debugger connected to your device, you can flash and run the firmware with gdb.
This requires the J-Link GDB server that is part of the [J-Link Software and Documentation Pack](https://www.segger.com/downloads/jlink/#J-LinkSoftwareAndDocumentationPack).

First, connect the debugger to the Nitrokey device, connect both to your computer and start the J-Link GDB server:

```
$ make jlink
```

Then execute gdb to run the firmware and enter the debugger:

```
$ make run
```

This is especially useful if logging via semihosting is enabled, e. g.:

```
$ make run FEATURES=log-semihosting,log-traceP
```

## Flashing the firmware

TODO: document

## Provisioning the firmware

TODO: document

## Resetting the device

TODO: document
