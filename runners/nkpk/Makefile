ALL_FEATURES := $(FEATURES),
BUILD_ID := nkpk
ARTIFACTS := artifacts
SYMBOLS := symbols-$(BUILD_ID).txt
OUT_ELF = $(ARTIFACTS)/runner-$(BUILD_ID).elf
OUT_BIN = $(ARTIFACTS)/runner-$(BUILD_ID).bin
OUT_IHEX = $(OUT_BIN).ihex

.PHONY: build
build:
	mkdir -p $(ARTIFACTS)
	cargo --version

	cargo build --release --features $(ALL_FEATURES)

	cargo objcopy --release --features $(ALL_FEATURES) --quiet -- $(OUT_ELF)

	arm-none-eabi-nm $(OUT_ELF) | grep ' [ABbDdRrTt] ' | sort | c++filt > $(SYMBOLS) 
	cat ../arm-none-eabi-arch-symbols.txt >> $(SYMBOLS)

	arm-none-eabi-objcopy $(OUT_ELF) -O binary ./$(OUT_BIN)
	arm-none-eabi-objcopy $(OUT_ELF) -O ihex ./$(OUT_IHEX)

.PHONY: check
check:
	cargo check --features $(ALL_FEATURES)

.PHONY: lint
lint:
	cargo clippy --features $(ALL_FEATURES)
