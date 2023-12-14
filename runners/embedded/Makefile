
CARGO_TARGET_DIR ?= ../../target

ifeq ($(BOARD), nk3am)
  SOC := nrf52
  TARGET := thumbv7em-none-eabihf
  LEGACY_BUILD_PROFILE := nrf52-bootloader
else ifeq ($(BOARD), nk3xn)
  SOC := lpc55
  TARGET := thumbv8m.main-none-eabi
  LEGACY_BUILD_PROFILE := lpc55
else ifneq ($(BOARD), )
  $(error Unsupported board $(BOARD))
endif


BUILD_ID ?= $(LEGACY_BUILD_PROFILE)-$(BOARD)

# gnu binutils-prefix
GNU_TARGET = arm-none-eabi

# util fn/paths
ARTIFACTS = artifacts
LOG ?= build-$(BUILD_ID).err.log
SYMBOLS ?= symbols-$(BUILD_ID).txt

# input files and outputs
SRCS = $(shell find src -name "*.rs" )
OUT_BIN = $(ARTIFACTS)/runner-$(BUILD_ID).bin
OUT_ELF = $(ARTIFACTS)/runner-$(BUILD_ID).elf
OUT_IHEX = $(OUT_BIN).ihex
CUSTOM_PROFILE=$(shell python3 -c "p = 'release-thin-lto' if '$(BOARD)' == 'nk3am' and 'test' in '$(COMMA_FEATURES)'.split(',')  else 'release'; print(p); " )
NO_DELOG_FEATURE=$(shell python3 -c "print('no-delog') if 'no-delog' not in '$(BUILD_FEATURES)'.split(',') and 'log-semihosting' not in '$(BUILD_FEATURES)'.split(',') and 'log-rtt' not in '$(BUILD_FEATURES)'.split(',') else None; ")
RAW_OUT = $(CARGO_TARGET_DIR)/$(TARGET)/$(CUSTOM_PROFILE)/$(SOC)_runner

# feature definition
BUILD_FEATURES := board-$(BOARD) $(FEATURES)
ALL_FEATURES = $(BUILD_FEATURES) $(NO_DELOG_FEATURE)
# assemble comma-seperated list to pass to `cargo build`
delim = ,
space := $(null) #
COMMA_FEATURES = $(subst $(space),$(delim),$(ALL_FEATURES))

.PHONY: list build reset program check doc check-all clean clean-all check-env set-vars

# default target -> just build all "shortcuts"
all: build-nk3am.bl build-nk3xn $(ARTIFACTS)

check-all: check-nk3am.bl check-nk3xn

$(ARTIFACTS):
	mkdir -p $@

help:
	###############################################################################
	###############################################################################
	## To build a specific 'board':
	## $ make build BOARD=nk3xn
	##
	## Or build using one of the pre-defined "target shortcuts" (see `make list`):
	## $ make build-nk3am.bl
	##
	## AND pass FEATURES as build feature-variants
	## $ make build-nk3am.bl FEATURES=complete
	###############################################################################
	###############################################################################

GET_TARGET = $(shell echo $(@) | cut -d '-' -f 1)

# shortcut targets
%-nk3am.bl: $(ARTIFACTS)
	$(MAKE) $(GET_TARGET) BOARD=nk3am FEATURES=$(FEATURES)
%-nk3xn: $(ARTIFACTS)
	$(MAKE) $(GET_TARGET) BOARD=nk3xn FEATURES=$(FEATURES)

$(OUT_ELF): $(SRCS) build
$(OUT_BIN): $(OUT_ELF)
$(OUT_IHEX): $(OUT_ELF)

check-var-%:
	@if [ -z '${${*}}' ]; then echo 'FAIL: var: $* required!!!' && exit 1; fi

%-banner:
	@echo "******************************************************************************************"
	@echo "**** TARGET:    $(shell printf %18s $(GET_TARGET)) | BINARY:   $(OUT_BIN)(.ihex)"
	@echo "**** BOARD:     $(shell printf %18s $(BOARD)) | SOC:      $(SOC)"
	@echo "**** BUILD_ID: $(BUILD_ID)"
	@echo "**** FEATURES:  $(BUILD_FEATURES)"
	@echo "******************************************************************************************"

list:
	@echo "Available:"
	@echo "  BOARD(s): $(shell find src/ -name 'board_*' | xargs -n1 basename | sed -E 's/board_([^.]*)(.rs)?/\1/g' | xargs)"
	@echo "  Makefile shortcuts: nk3xn, nk3am.bl"
	@echo ""
	@echo "  Usage: make <target>-<something> e.g., build-nk3am.bl, reset-nk3xn, ..." 

clean-all: 
	rm -f ./$(OUT_BIN) ./$(OUT_IHEX) $(SYMBOLS) $(LOG)
	rm -f *.log runner-*-*.ihex runner-*-*.bin symbols-*-*.txt runner-*-*.ihex
	rm -rf ./$(CARGO_TARGET_DIR)
	rm -f firmware.hex mbr.hex bootloader.hex
	rm -rf artifacts cfg.toml

###############################################################################
#### actual build, clean, reset, program targets
###############################################################################

build: build-banner $(SRCS) check-var-BOARD check-var-SOC

	cargo --version

	# NRF52/test -> "release-thin-lto", use "release" otherwise
	cargo build --target $(TARGET) \
		--features $(COMMA_FEATURES) \
		--quiet --profile $(CUSTOM_PROFILE)

	cp $(RAW_OUT) ./$(OUT_ELF)

	$(GNU_TARGET)-nm ./$(OUT_ELF) | grep ' [ABbDdRrTt] ' | sort | c++filt > $(SYMBOLS) 
	
	cat $(GNU_TARGET)-arch-symbols.txt >> $(SYMBOLS)

	$(GNU_TARGET)-objcopy -O binary ./$(OUT_ELF) ./$(OUT_BIN)
	$(GNU_TARGET)-objcopy -O ihex ./$(OUT_ELF) ./$(OUT_IHEX)
	$(GNU_TARGET)-readelf -l ./$(OUT_ELF) | grep LOAD

check: check-banner $(SRCS) check-var-BOARD check-var-SOC

	cargo --version

	cargo check --target $(TARGET) \
		--features $(COMMA_FEATURES) \
		--quiet --profile $(CUSTOM_PROFILE)

doc: check-banner $(SRCS) check-var-BOARD check-var-SOC

	cargo --version

	cargo doc --target $(TARGET) \
		--features $(COMMA_FEATURES) \
		--verbose

clean: clean-banner check-var-BOARD
	rm -f ./$(OUT_BIN) ./$(OUT_ELF) ./$(OUT_IHEX) $(RAW_OUT) $(SYMBOLS) $(LOG)

