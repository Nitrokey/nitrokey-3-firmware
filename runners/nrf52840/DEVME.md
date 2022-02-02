

* build: `make`

* flash: `make reset-nrfdk`

* start debugging with openocd: `openocd -f ocd.conf`

ocd.conf:
```
source [find interface/jlink.cfg]
jlink serial 000683471542

transport select swd
source [find target/nrf52.cfg]
```

* get backtrace using gdb: `arm-none-eabi-gdb --batch -x gdb.cmds`

gdb.cmds:
```
target extended-remote localhost:3333
file target/thumbv7em-none-eabihf/release/runner-nrfdk
bt
```
* get "raw" backtrace: `python ocdtool.py Dhf`

* get `debug!("foo")` style debug information by using 
  
	* `python ocdtool.py Dr` to setup RTT
	* `nc localhost 4999` to dump the messages (you might want to `grep -v 'irq RTC'`)





