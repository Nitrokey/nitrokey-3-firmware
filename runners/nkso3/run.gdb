target extended-remote :3333
monitor reset halt
load

# setup vector table: SCB->VTOR = .vector_table
mem 0xE0000000 0xE00FFFFF
set *0xE000ED08 = 0x34064000

jump Reset
