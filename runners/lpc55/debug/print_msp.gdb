set history save on
set confirm off
set pagination off

define lmsp
  while (1)
    # f
    i r msp
    step 100
  end
end


# dir src/
target extended-remote :2331
# file runner-lpc55-nk3xn.elf
# load runner-lpc55-nk3xn.elf
monitor reset
# file artifacts/runner-lpc55-nk3xn.elf
source print_msp.py