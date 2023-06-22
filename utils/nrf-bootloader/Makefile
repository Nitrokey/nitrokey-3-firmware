
IMAGE_NAME := nrf-toolchain 

.PHONY: all enter clean verify

all: $(IMAGE_NAME).built verify
	docker run -it \
		-v $(shell pwd):/work \
		$(IMAGE_NAME) \
		make -f Makefile.docker

enter: $(IMAGE_NAME).built
	docker run -it \
		-v $(shell pwd):/work \
		$(IMAGE_NAME) \
		bash

verify: $(IMAGE_NAME).built
	docker run -it \
		-v $(shell pwd):/work \
		$(IMAGE_NAME) \
		sha256sum -c /work/sha256sums.txt	
	
$(IMAGE_NAME).built: Dockerfile
	docker build -t $(IMAGE_NAME) .
	touch $@

clean:
	rm -rf build
	rm -f mbr.hex bootloader.hex $(IMAGE_NAME).built output.zip

############################################
############################################  NRF52 Bootloader stuff
############################################

.PHONY: fw-assemble-dfu fw-upload-dfu generate-key

FIRMWARE_VERSION = 15

BL_DIR = .
BL_HEX = $(BL_DIR)/bootloader.hex
MBR_HEX = $(BL_DIR)/mbr.hex
BL_CONFIGS = $(BL_DIR)/config/default_sdk_config.h $(BL_DIR)/config/nk_sdk_config.h $(BL_DIR)/config/sdk_config.h

# descent to build mbr.hex + booloader.hex 
$(BL_HEX) $(MBR_HEX): $(BL_DIR) $(BL_CONFIGS)
	$(MAKE) -C $(BL_DIR)


KEY_DIR ?= $(BL_DIR)/signing-key
SIGNING_KEY = $(KEY_DIR)/dfu_private.key
PUBLIC_KEY = $(KEY_DIR)/dfu_public_key.c

SIGN_SH = $(BL_DIR)/sign.sh
UPLOAD_SH = $(BL_DIR)/upload.sh

FW_SIGNED = fw_signed_update.zip
FW_RAW = fw_unsigned.hex
TTYDEV := $(wildcard /dev/serial/by-id/usb-Nitrokey_Nitrokey_3_Bootloader*)
# /dev/serial/by-id/usb-Nitrokey_Nitrokey_3-if02
# /dev/serial/by-id/usb-Nitrokey_Nitrokey_3_Bootloader_F95AF80A9E98-if00

fw-assemble-dfu: $(FW_SIGNED)

$(FW_RAW): $(OUT_IHEX)
	cp $< $(@)

$(FW_SIGNED): $(FW_RAW) $(SIGN_SH) # $(SIGNING_KEY) 
	bash $(SIGN_SH) $(FIRMWARE_VERSION) $(FW_SIGNED) $(FW_RAW) $(SIGNING_KEY)

sign-nk3am.bl: $(FW_SIGNED)
	

fw-upload: $(FW_SIGNED) $(UPLOAD_SH) 
	-if which nitropy; then \
		nitropy nk3 reboot --bootloader; \
	fi
	bash $(UPLOAD_SH) $(FW_SIGNED) $(TTYDEV)

generate-key:
	@echo "----------- THIS IS NOT AN AUTOMATED STEP ----------"
	@echo "Before following these steps, make sure you are not overwriting a key!"
	@echo "To generate a proper key-pair run the following command:"
	@echo "$ generate_key.sh <target_directory>"

$(SIGNING_KEY): generate-key
	@echo "NO SIGNING KEY FOUND...."
	@echo "GENERATE one and put it here: $(SIGNING_KEY)"
	exit 1


lazy-fw:
	$(MAKE) fw-assemble-dfu
	$(MAKE) fw-upload-dfu TTYDEV=`ls -1rt | tail -n1`


build-bootloader: $(BOOTLOADER_HEX) $(MBR_HEX)

reset-bootloader: $(BOOTLOADER_HEX) $(MBR_HEX)
	nrfjprog -f NRF52 --recover
	nrfjprog -f NRF52 --eraseall
	nrfjprog -f NRF52 --program mbr.hex --sectorerase --verify
	nrfjprog -f NRF52 --program bootloader.hex --sectorerase --verify
	
	# UICR
	## UICR::REGOUT0 to 3v3
	nrfjprog -f NRF52 --memwr 0x10001304 --val 0xfffffffd --verify
	## UICR::NFCPINS to disabled
	nrfjprog -f NRF52 --memwr 0x1000120C --val 0xfffffffe --verify
	## UICR::APPROTECT (only --recover will revive debug-access)
	#nrfjprog -f NRF52 --memwr 0x10001208 --val 0xFFFFFF00
	
	nrfjprog -f NRF52 --reset

program-bootloader:	$(BOOTLOADER_HEX) $(MBR_HEX)
	nrfjprog -f NRF52 --program mbr.hex --sectorerase --verify
	nrfjprog -f NRF52 --program bootloader.hex --sectorerase --verify
	nrfjprog -f NRF52 --reset




