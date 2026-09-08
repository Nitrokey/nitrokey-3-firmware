target extended-remote :3333
set mem inaccessible-by-default off
monitor reset halt
load

# SCB->VTOR = .vector_table (start of the FLASH region in AXISRAM1)
mem 0xE0000000 0xE00FFFFF
set language c
set *(unsigned int *)0xE000ED08 = 0x34064000
set language auto

# *Reset: plain "jump Reset" skips the prologue and with it the MSP setup
jump *Reset
