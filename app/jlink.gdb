# source .gdb-dashboard

set history save on
set confirm off

# find commit-hash using `rustc -Vv`
set substitute-path /rustc/5e1a799842ba6ed4a57e91f7ab9435947482f7d8 /home/nicolas/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust

target extended-remote :2331
load
monitor reset
monitor semihosting enable
# monitor semihosting breakOnError <digit>
# by default (1) output goes to Telnet client, 2 sends to GDB client, 3 would send to both
monitor semihosting IOClient 3
#break led.rs:67
#continue
#monitor swo enabletarget 0 0 1 0
# mon SWO EnableTarget 0 48000000 1875000 0
continue
# stepi
