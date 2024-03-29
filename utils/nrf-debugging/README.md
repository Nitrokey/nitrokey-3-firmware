### This directory contains a collection of (nrf52) debugging tools and helpers

Generally all the following descriptions refer to a flashed device, 
which is connected to a debugger (nRF52840-DK). Make sure you first
connect the device to the debugger and *afterwards* power on the debugger.

* Get the correct serial number for `ocd.conf` using `nrfjprog -i`, then use the template in `ocd.conf.example` to create the correct `ocd.conf` (Otherwise you can use `make ocd.conf` with the debugger plugged in).

* How to show RTT (print) debugging outputs:

```
# - flash the device (make -C ../nrf-builder flash-develop)

# inside a terminal, connect to the debugger
make ocd

# inside another terminal, show rtt outputs
make rtt
```

* How to run `gdb`
```
make ocd
make it
```

*  How to dump the filesystem
```
make lfs_fast

# after this operation the device is stuck!
```


