
TEST_CERTS_REPO := git@git.nitrokey.com:robin/test-certificates.git
TEST_CERTS_DIR := test-certs
TEST_FIDO_CERTS_DIR := $(TEST_CERTS_DIR)/nk3/fido
TEST_FIDO_CERT := $(TEST_FIDO_CERTS_DIR)/nk-fido-ee-cert.der
TEST_FIDO_KEY := $(TEST_FIDO_CERTS_DIR)/nk-fido-ee-key.trussed

NK3_FW_REPO := git@github.com:Nitrokey/nitrokey-3-firmware.git
NK3_FW_DIR := nk3-fw
NK3_FW_BRANCH := embedded-pr

NK3_BL_DIR := $(NK3_FW_DIR)/utils/nrf-bootloader
NK3_BL_KEY_DIR := $(NK3_FW_DIR)/utils/nrf-bootloader/signing-key

KEY_DIR := $(TEST_CERTS_DIR)/nk3/firmware-nrf52
PRIV_SIGN_KEY := $(KEY_DIR)/dfu_private.pem

BL_OUTPUT := mbr.hex bootloader.hex

FW_RUNNER := $(NK3_FW_DIR)/runners/embedded
FW_ARTIFACTS := $(FW_RUNNER)/artifacts

CARGO_TOML_PATH := $(NK3_FW_DIR)/Cargo.toml

FW_IHEX := $(FW_ARTIFACTS)/runner-nrf52-bootloader-nk3am.bin.ihex
FW_BIN := $(FW_ARTIFACTS)/runner-nrf52-bootloader-nk3am.bin

SH_SIGN := $(NK3_BL_DIR)/sign.sh
SH_UPLOAD := $(NK3_BL_DIR)/upload.sh
SH_BL_SIGN := $(NK3_BL_DIR)/sign-bootloader.sh

REPOS := $(TEST_CERTS_DIR)-update

FW_VERSION := $(shell grep 'version' $(CARGO_TOML_PATH) | head -n1 | grep -o '[0-9\.]*')
FW_VERSION_MAJOR := $(shell echo $(FW_VERSION) | cut -d "." -f 1)
FW_VERSION_MINOR := $(shell echo $(FW_VERSION) | cut -d "." -f 2)
FW_VERSION_PATCH := $(shell echo $(FW_VERSION) | cut -d "." -f 3)
SIGN_VERSION := $(shell python -c "print( ($(FW_VERSION_MAJOR) << 22) | ($(FW_VERSION_MINOR) << 6) | ($(FW_VERSION_PATCH)) )")

FW_NAME_PROVISIONER_HEX := provisioner-nk3am-nrf52-$(FW_VERSION).hex
FW_NAME_PROVISIONER_BIN := provisioner-nk3am-nrf52-$(FW_VERSION).bin
FW_NAME_PROVISIONER := provisioner-nk3am-nrf52-$(FW_VERSION).zip

FW_NAME_DEVELOP_HEX := develop-nk3am-nrf52-$(FW_VERSION).hex
FW_NAME_DEVELOP_BIN := develop-nk3am-nrf52-$(FW_VERSION).bin
FW_NAME_DEVELOP := develop-nk3am-nrf52-$(FW_VERSION).zip

FW_NAME_RELEASE_HEX := firmware-nk3am-nrf52-$(FW_VERSION).hex
FW_NAME_RELEASE_BIN := firmware-nk3am-nrf52-$(FW_VERSION).bin
FW_NAME_RELEASE := firmware-nk3am-nrf52-$(FW_VERSION).zip

BL_SIGNED_NAME := bootloader-nk3am-nrf52-$(FW_VERSION).zip

ALL_ARTIFACTS := $(FW_NAME_RELEASE) $(FW_NAME_PROVISIONER) $(FW_NAME_DEVELOP) \
                 $(BL_OUTPUT) $(BL_SIGNED_NAME) \
								 $(FW_NAME_DEVELOP_HEX) $(FW_NAME_PROVISIONER_HEX) $(FW_NAME_RELEASE_HEX) \
								 $(FW_NAME_DEVELOP_BIN) $(FW_NAME_PROVISIONER_BIN) $(FW_NAME_RELEASE_BIN) 

SRCS = $(shell find $(FW_RUNNER)/src -name "*.rs" )

TTY := $(shell ls -1rt /dev/ttyACM* | tail -n 1 | xargs)

EXTRA_FEATURES :=


######

.PHONY: build clean flash-provisioner flash-fw flash-bootloader provision-keys full-deploy

build: $(ALL_ARTIFACTS)
	@echo "#########################################################"
	@echo "####################### Build complete:"
	@echo "Artifacts:"
	@ls --block-size 1 --size -1 $(ALL_ARTIFACTS) | grep hex | sort -h | sed 's/\</\  /'
	@echo
	@ls --block-size 1 --size -1 $(ALL_ARTIFACTS) | grep bin | sort -h | sed 's/\</\  /'
	@echo
	@ls --block-size 1 --size -1 $(ALL_ARTIFACTS) | grep zip | sort -h | sed 's/\</\  /'
	@echo ""
	@echo "Versions:"
	@echo "- cargo.toml: $(FW_VERSION)"
	@echo "- signing: $(SIGN_VERSION)"
	@echo 
	@echo "Git:"
	@echo	"- commit: $(shell git rev-parse --short HEAD)"
	@echo "- describe: $(shell git describe)"
	@echo "- active branch: $(shell git branch -l | grep "*" | cut -d ' ' -f 2)"
	@echo "#########################################################"

flash-release: $(FW_NAME_RELEASE) $(SH_UPLOAD)
	-nitropy nk3 reboot --bootloader
	./wait_for_usb.sh 20a0:42e8
	
	$(SH_UPLOAD) $(FW_NAME_RELEASE) $(TTY)
	
	./wait_for_usb.sh 20a0:42b2

flash-develop: $(FW_NAME_DEVELOP) $(SH_UPLOAD)
	-nitropy nk3 reboot --bootloader
	./wait_for_usb.sh 20a0:42e8
	
	$(SH_UPLOAD) $(FW_NAME_DEVELOP) $(TTY)
	
	./wait_for_usb.sh 20a0:42b2

flash-provisioner: $(FW_NAME_PROVISIONER) $(SH_UPLOAD)
	-nitropy nk3 reboot --bootloader
	./wait_for_usb.sh 20a0:42e8
	
	$(SH_UPLOAD) $(FW_NAME_PROVISIONER) $(TTY)
	
	./wait_for_usb.sh 20a0:42b2

flash-bootloader: $(BL_OUTPUT)
	$(MAKE) -C $(NK3_BL_DIR) reset-bootloader
	./wait_for_usb.sh 20a0:42e8

provision-keys: $(TEST_FIDO_CERT) $(TEST_FIDO_KEY)
	./wait_for_usb.sh 20a0:42b2
	solo2 app provision write-file $(TEST_FIDO_CERT) fido/x5c/00
	solo2 app provision write-file $(TEST_FIDO_KEY) fido/sec/00

full-deploy:
	$(MAKE) flash-bootloader
	$(MAKE) flash-provisioner
	$(MAKE) provision-keys
	$(MAKE) flash-release

fast-deploy:
	$(MAKE) flash-bootloader
	$(MAKE) flash-release

clean:
	rm -rf $(ALL_ARTIFACTS)
	rm -rf test-certs 


######

$(TEST_CERTS_DIR):
	git clone $(TEST_CERTS_REPO) $@

.PHONY: $(TEST_CERTS_DIR)-update
$(TEST_CERTS_DIR)-update: $(TEST_CERTS_DIR)
	cd $(TEST_CERTS_DIR) && git pull

$(KEY_DIR) $(PRIV_SIGN_KEY): $(TEST_CERTS_DIR)-update

$(NK3_BL_KEY_DIR): $(KEY_DIR) $(REPOS)
	cp -r $< $(NK3_BL_KEY_DIR)

$(BL_OUTPUT): $(NK3_FW_DIR) $(NK3_BL_KEY_DIR) $(REPOS)
	$(MAKE) -C $(NK3_BL_DIR) build-bootloader
	cp $(NK3_BL_DIR)/mbr.hex mbr.hex
	cp $(NK3_BL_DIR)/bootloader.hex bootloader.hex

$(FW_RUNNER): $(NK3_FW_DIR) $(REPOS)

$(BL_SIGNED_NAME): bootloader.hex $(PRIV_SIGN_KEY)
	$(SH_BL_SIGN) $(SIGN_VERSION) $@ $< $(PRIV_SIGN_KEY)

%.zip: %.hex $(PRIV_SIGN_KEY)
	$(SH_SIGN) $(SIGN_VERSION) $@ $< $(PRIV_SIGN_KEY)

$(FW_NAME_PROVISIONER_HEX) $(FW_NAME_PROVISIONER_BIN): $(NK3_FW_DIR) $(SRCS)
	$(MAKE) -C $(FW_RUNNER) build-nk3am.bl FEATURES=provisioner,$(EXTRA_FEATURES)
	cp $(FW_IHEX) $(FW_NAME_PROVISIONER_HEX)
	cp $(FW_BIN) $(FW_NAME_PROVISIONER_BIN)

$(FW_NAME_DEVELOP_HEX) $(FW_NAME_DEVELOP_BIN): $(NK3_FW_DIR) $(SRCS)
	$(MAKE) -C $(FW_RUNNER) build-nk3am.bl FEATURES=develop,$(EXTRA_FEATURES)
	cp $(FW_IHEX) $(FW_NAME_DEVELOP_HEX)
	cp $(FW_BIN) $(FW_NAME_DEVELOP_BIN)

$(FW_NAME_RELEASE_HEX) $(FW_NAME_RELEASE_BIN): $(NK3_FW_DIR) $(SRCS)
	$(MAKE) -C $(FW_RUNNER) build-nk3am.bl FEATURES=release,$(EXTRA_FEATURES)
	cp $(FW_IHEX) $(FW_NAME_RELEASE_HEX)
	cp $(FW_BIN) $(FW_NAME_RELEASE_BIN)

