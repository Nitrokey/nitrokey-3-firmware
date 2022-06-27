# USB/IP Simulation

This runner allows using USB/IP as a means to simulate device connection
to the OS, and should allow faster development of the embedded applications.

Platform and storage implementations are taken from the Trussed tutorial:
- https://github.com/trussed-dev/trussed-totp-pc-tutorial

Remarks:
- At the moment FIDO app only (to be extended with Admin and Provision apps);
- Works with Chromium and pynitrokey (with a patched fido2.hid module) [2];
- Written length returns "1", which confuse client HID applications
  (Chromium shows error in logs, but ignores it; pynitrokey fails);
- It is not possible to set the FIDO certificate, thus x5c response
  is empty;
- Does not work with Firefox at the moment;
- Requires multiple `usbip attach` calls to make it work [1].

[1] https://github.com/Sawchord/usbip-device#known-bugs

[2] The change is rather simple: replace `raise OSError("failed to write entire packet")` with `pass` in `FileCtapHidConnection.write_packet` in fido2’s `hid/base.py`. The patch is provided at [3].

[3] ./fido2-patch/0001-Ignore-difference-between-the-sent-data-size-and-rep.patch

## Setup

USB/IP tools are required to work, as well as kernel supporting it.

On Fedora these could be installed with:
```
make setup-fedora
```

## Run 

Simulation starts USB/IP server, which can be connected to with the USB/IP tools. 
1. Make sure `vhci-hcd` module is loaded
2. Run simulation app
3. Attach to the simulated device (2 times if needed) 

This series of steps is scripted in the Makefile, thus it is sufficient to call:
```
make 
```

Stop execution with:
``` 
make stop
```

Warning: in some cases simulation can sometimes cause kernel faults, which makes the system it is running unstable.
