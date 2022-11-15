### This directory contains a collection of (nrf52) debugging tools and helpers

Generally all the following descriptions refer to a flashed device, 
which is connected to a debugger (nRF52840-DK). Make sure you first
connect the device to the debugger and *afterwards* power on the debugger.


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
make gdb
```

*  How to dump the filesystem
```
make lfs_fast

# after this operation the device is stuck!
```


