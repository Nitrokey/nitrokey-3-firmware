

BUILD_ID ?= $(BUILD_PROFILE)-$(BOARD)
CFG_PATH ?= profiles/$(BUILD_PROFILE).toml

CARGO_TARGET_DIR ?= ../../target

ifeq ($(filter nk3am proto1 nrfdk,$(BOARD)), )
  SOC = lpc55
else
  SOC = nrf52
endif

# rust triplet
TARGET = $(shell python3 -c 'import toml; print(toml.load("$(CFG_PATH)")["platform"]["target"])')
# gnu binutils-prefix
GNU_TARGET = arm-none-eabi

### @TODO: ugly deluxe ? toml bash parser? dirty grep? 
CFG_VAL_BOARDS = $(shell python3 -c 'import toml; print(" ".join(toml.load("$(CFG_PATH)")["platform"]["boards"]))')
CFG_VAL_SOC = $(shell python3 -c 'import toml; print(toml.load("$(CFG_PATH)")["platform"]["soc"])')

# util fn/paths
ARTIFACTS = artifacts
LOG ?= build-$(BUILD_ID).err.log
SYMBOLS ?= symbols-$(BUILD_ID).txt

# input files and outputs
SRCS = $(shell find src -name "*.rs" )
OUT_BIN = $(ARTIFACTS)/runner-$(BUILD_ID).bin
OUT_ELF = $(ARTIFACTS)/runner-$(BUILD_ID).elf
OUT_IHEX = $(OUT_BIN).ihex
RAW_OUT = $(CARGO_TARGET_DIR)/$(TARGET)/release/$(SOC)_runner

# feature definition
BUILD_FEATURES := board-$(BOARD) $(FEATURES)
# assemble comma-seperated list to pass to `cargo build`
delim = ,
space := $(null) #
COMMA_FEATURES = $(subst $(space),$(delim),$(BUILD_FEATURES))

.PHONY: list build reset program check check-all clean clean-all check-env set-vars

# default target -> just build all "shortcuts"
all: build-nrfdk build-nk3am build-proto1 build-nk3xn $(ARTIFACTS)

check-all: check-nrfdk check-nk3am check-proto1 check-nk3xn

$(ARTIFACTS):
	mkdir -p $@

help:
	###############################################################################
	###############################################################################
	## To build a specific 'board' & 'build_profile' combination:
	## $ make build BUILD_PROFILE=blubla BOARD=foobar
	##
	## Or build using one of the pre-defined "target shortcuts" (see `make list`):
	## $ make build-proto1
	##
	## AND pass FEATURES as build feature-variants
	## $ make build-nk3am FEATURES=complete
	###############################################################################
	###############################################################################

GET_TARGET = $(shell echo $(@) | cut -d '-' -f 1)

# shortcut targets
%-nrfdk: $(ARTIFACTS)
	$(MAKE) $(GET_TARGET) BUILD_PROFILE=nrf52 BOARD=nrfdk FEATURES=$(FEATURES)
%-nk3am: $(ARTIFACTS)
	$(MAKE) $(GET_TARGET) BUILD_PROFILE=nrf52 BOARD=nk3am FEATURES=$(FEATURES)
%-nk3am.bl: $(ARTIFACTS)
	$(MAKE) $(GET_TARGET) BUILD_PROFILE=nrf52-bootloader BOARD=nk3am FEATURES=$(FEATURES)
%-proto1: $(ARTIFACTS)
	$(MAKE) $(GET_TARGET) BUILD_PROFILE=nrf52 BOARD=proto1 FEATURES=$(FEATURES)
%-nk3xn: $(ARTIFACTS)
	$(MAKE) $(GET_TARGET) BUILD_PROFILE=lpc55 BOARD=nk3xn FEATURES=$(FEATURES)


ifeq ($(BUILD_ID),)
ifneq ($(shell stat $(CFG_PATH) | head -n1),File: $(CFG_PATH))
 $(error Configuration: $(CFG_PATH) not found!)
 $(shell exit 1)
endif
endif

$(OUT_ELF): $(SRCS) build
$(OUT_BIN): $(OUT_ELF)
$(OUT_IHEX): $(OUT_ELF)

check-var-%:
	@if [ -z '${${*}}' ]; then echo 'FAIL: var: $* required!!!' && exit 1; fi

%-banner:
	@echo "******************************************************************************************"
	@echo "**** TARGET:    $(shell printf %18s $(GET_TARGET)) | BINARY:   $(OUT_BIN)(.ihex)"
	@echo "**** BOARD:     $(shell printf %18s $(BOARD)) | SOC:      $(SOC)"
	@echo "**** PROFILE:   $(shell printf %18s $(BUILD_PROFILE)) | BUILD_ID: $(BUILD_ID)"
	@echo "**** FEATURES:  $(BUILD_FEATURES)"
	@echo "******************************************************************************************"

list:
	@echo "Available:"
	@echo "  BOARD(s): $(shell find src/ -name 'board_*' | xargs -n1 basename | sed -E 's/board_([^.]*)(.rs)?/\1/g' | xargs)"
	@echo "  BUILD_PROFILE(s): $(shell ls profiles -1 | sed -e 's/\.toml//g')"
	@echo "  Makefile shortcuts: nrfdk, proto1, nk3xc, nk3am, nk3am.bl"
	@echo ""
	@echo "  Usage: make <target>-<something> e.g., build-nk3am, reset-proto1, ..." 

clean-all: 
	rm -f ./$(OUT_BIN) ./$(OUT_IHEX) $(SYMBOLS) $(LOG)
	rm -f *.log runner-*-*.ihex runner-*-*.bin symbols-*-*.txt runner-*-*.ihex
	rm -rf ./$(CARGO_TARGET_DIR)
	rm -f firmware.hex mbr.hex bootloader.hex
	rm -rf artifacts cfg.toml

###############################################################################
#### actual build, clean, reset, program targets
###############################################################################

build: build-banner $(SRCS) check-var-BOARD check-var-BUILD_PROFILE check-var-SOC

	cargo --version

	cp -f $(CFG_PATH) cfg.toml
	echo '' >> cfg.toml
	echo '[build]' >> cfg.toml
	echo 'build_profile = "$(BUILD_PROFILE)"' >> cfg.toml
	echo 'board = "$(BOARD)"' >> cfg.toml

	cargo build --release --target $(TARGET) \
		--features $(COMMA_FEATURES) \
		--quiet

	cp $(RAW_OUT) ./$(OUT_ELF)

	$(GNU_TARGET)-nm ./$(OUT_ELF) | grep ' [ABbDdRrTt] ' | sort | c++filt > $(SYMBOLS) 
	
	cat $(GNU_TARGET)-arch-symbols.txt >> $(SYMBOLS)

	$(GNU_TARGET)-objcopy -O binary ./$(OUT_ELF) ./$(OUT_BIN)
	$(GNU_TARGET)-objcopy -O ihex ./$(OUT_ELF) ./$(OUT_IHEX)
	$(GNU_TARGET)-readelf -l ./$(OUT_ELF) | grep LOAD

check: check-banner $(SRCS) check-var-BOARD check-var-BUILD_PROFILE check-var-SOC

	cargo --version

	cp -f $(CFG_PATH) cfg.toml
	echo '' >> cfg.toml
	echo '[build]' >> cfg.toml
	echo 'build_profile = "$(BUILD_PROFILE)"' >> cfg.toml
	echo 'board = "$(BOARD)"' >> cfg.toml

	cargo check --release --target $(TARGET) \
		--features $(COMMA_FEATURES) \
		--quiet

clean: clean-banner check-var-BOARD check-var-BUILD_PROFILE
	rm -f ./$(OUT_BIN) ./$(OUT_ELF) ./$(OUT_IHEX) $(RAW_OUT) $(SYMBOLS) $(LOG)

