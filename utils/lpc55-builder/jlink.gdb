set history save on
set confirm off
set pagination off

define rebootloop
  while (1)
    run
  end
end

target extended-remote :2331
load
monitor reset
monitor semihosting enable
monitor semihosting IOClient 3

# to add a breakpoint, use something like:
# break led.rs:67

continue
