BOARD ?= nk3xn

FEATURES := board-${BOARD}
DEVELOP := 1

FLAGS=

ifeq "${DEVELOP}" "1"
FEATURES := ${FEATURES},develop
FLAGS=
else
FLAGS := --release
endif

ifeq "${PROVISIONER}" "1"
FEATURES := ${FEATURES},develop-provisioner
endif

all: $(BINARY)

.PHONY: build
build:
	cargo build $(FLAGS) --features $(FEATURES) --verbose

.PHONY: run
run:
	cargo run $(FLAGS) --features $(FEATURES)

.PHONY: $(BINARY)
BINARY=firmware-${BOARD}.bin
$(BINARY):
	cargo objcopy $(FLAGS) --features $(FEATURES) -- -O binary $@

.PHONY: objcopy
objcopy:
	cargo objcopy --features $(FEATURES) --verbose -- -O binary $(BINARY) --verbose

.PHONY: flash
flash: $(BINARY)
	mboot erase --mass
	mboot write $<

.PHONY: size
size:
	cargo size $(FLAGS) --features $(FEATURES)

.PHONY: bacon
bacon:
	bacon

.PHONY: jlink
jlink:
	-../../scripts/bump-jlink
	JLinkGDBServer -strict -device LPC55S69 -if SWD -vd


# TODO this is executed each Makefile parsing
PACK_VERSION := $(shell wget -O - -qq https://mcuxpresso.nxp.com/cmsis_pack/repo/NXP.pidx|grep LPC55S69|python -c'import sys; print(sys.stdin.read().rsplit("version=\"", 1)[1].split("\"", 1)[0])')
PACK := NXP.LPC55S69_DFP.$(PACK_VERSION).pack
get-cmsis-pack:
	wget -qq https://mcuxpresso.nxp.com/cmsis_pack/repo/$(PACK) -O ./lpc55s69.pack

.PHONY: clean
clean:
	cargo clean