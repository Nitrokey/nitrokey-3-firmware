# USB/IP Guide

This guide introduces the USB/IP runner and shows how it can be used to simulate a Nitrokey 3 device.

## Overview

[USB/IP][] is a protocol that makes it possible to simulate USB devices over IP packets.
We use it to simulate Nitrokey 3 devices in software.
While this means that many low-level parts of the firmware are replaced with the Rust standard library, the host operating system and USB/IP, it still allows easy development and testing of high-level components like the FIDO2 implementation.

[USB/IP]: https://usbip.sourceforge.net/

## Requirements

- GNU Make
- Git
- Rust
  - See the [`rust-toolchain.toml`][] file in the repository root for the required version.
- usbip
  - While USB/IP is also available for other operating systems, we have only tested and used it on Linux.

[`rust-toolchain.toml`]: ../rust-toolchain.toml

## Running the Simulation

Clone the firmware repository [Nitrokey/nitrokey-3-firmware](https://github.com/Nitrokey/nitrokey-3-firmware) and enter the `runners/usbip` directory:

```
$ git clone https://github.com/Nitrokey/nitrokey-3-firmware.git
$ cd runners/usbip
```

Build the USB/IP runner:

```
$ cargo build
```

Execute the USB/IP runner:

```
$ cargo run
```

This starts the USB/IP device.
Now we have to attach the device, i. e. tell USB/IP to enable the simulated device on the host.
The following command requires root privileges (using `sudo`), loads the `vhci-hcd` kernel module and attaches the device.
You should review it before executing and adapt if necessary.

```
$ make attach
```

Now the device should show up in `lsusb` and you can use e. g. it with `nitropy`.

## Configuration

### Logging

Use the `RUST_LOG` environment variable to activate log messages.
It uses the `env_logger` syntax, see the [`env_logger` documentation][].
For example, you could use `RUST_LOG=info,fido_authenticator=trace` to enable all error, warn and info messages as well as the debug and trace messages from `fido-authenticator`.
Note that some log message use the `!` target so you might not be able to filter them as expected.

[`env_logger` documentation]: https://docs.rs/env_logger/latest/env_logger/

### Command-Line Options

- Per default, the runner puts the internal and external filesystem into RAM.
  If you want to write it to a file instead, for example to inspect it afterwards or to persist it between runs, pass a file name to the `--ifs` and/or `--efs` options.
- Per default, the serial number of the device is generated randomly.
  You can set a fixed serial number using the `--serial` option.
- On hardware, user presence checks are implemented using the touch button.
  You can use the `--user-presence` option to define how the simulation responds to these requests:
  - `accept-all` (default) always accepts user presence checks.
  - `reject-all` always rejects user presence checks.
  - `interactive` shows a query on stderr when a user presence check is executed.
  - `signal` accepts the next user presence check within one second after receiving a SIGUSR1 signal, e. g. with `pkill -SIGUSR1 usbip-runner`.

For more information on these options, execute `cargo run -- --help`.

## Limitations

The Nitrokey 3 implements two transport protocols over USB: CTAPHID and CCID.
There is an unresolved issue that triggers a kernel bug if the CCID transport is used with the USB/IP runner ([#261][]).
Therefore CCID is disabled by default and it is recommended to only use the USB/IP runner with the CTAPHID transport.
Applications like [`opcard`][] support an alternative simulation method, `vsmartcard`, to reliably simulate the CCID transport.

[#261]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/261
[`opcard`]: https://github.com/Nitrokey/opcard-rs

If you really want to use CCID with the USB/IP runner, activate the `ccid` feature.
To avoid accidentally triggering this problem, it is recommended to stop `pcsc` before starting the USB/IP simulation if the `ccid` feature is activated.
```
$ sudo systemctl stop pcscd.service pcscd.socket
$ cargo run --features ccid
```
