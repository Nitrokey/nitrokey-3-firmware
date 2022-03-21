SERIAL_NRFDK := 683471542
SERIAL_PROTO1 := 801035168

#DEVICE_IDS_PRESENT := $(shell nrfjprog -i)
#DEVICES_PRESENT := $(if $(findstring $(SERIAL_NRFDK),$(DEVICE_IDS_PRESENT)),nrfdk,)
#DEVICES_PRESENT += $(if $(findstring $(SERIAL_PROTO1),$(DEVICE_IDS_PRESENT)),proto1,)
#DEVICES_PRESENT += nrfdk
DEVICES_PRESENT := nk3mini

$(info Connected devices: $(DEVICES_PRESENT))

OUTDIR2 := target/thumbv7em-none-eabihf

.PHONY: build-all build-release build-debug
.PHONY: build-nrfdk-release build-nrfdk-debug build-proto1-release build-proto1-debug
.PHONY: clean update deny
.PHONY: program program-nrfdk program-proto1
.PHONY: reset reset-nrfdk reset-proto1

build-release: $(patsubst %,build-%-release,$(DEVICES_PRESENT))

build-debug: $(patsubst %,build-%-debug,$(DEVICES_PRESENT))

build-all: build-release build-debug

program: $(patsubst %,program-%,$(DEVICES_PRESENT))

reset: $(patsubst %,reset-%,$(DEVICES_PRESENT))

ifeq ($(EXTRA_FEATURES),)
FEAT := 
else
FEAT := --features $(EXTRA_FEATURES)
endif

define build-template
build-$(1)-$(2):
	@echo "[31;1mBuilding board [$(1)] flavor [$(2)]...[37;0m"
	rm -f $(OUTDIR2)/$(2)/runner-$(1)
	cargo build $$(if $$(findstring release,$(2)),--release) --features board-$(1) $(FEAT) --color always 2> cargo_stderr.log; cat cargo_stderr.log
	arm-none-eabi-nm $(OUTDIR2)/$(2)/runner | grep ' [ABbDdRrTt] ' | sort | c++filt > symbols-$(1)-$(2).txt
	cat arm-special-symbols.txt >> symbols-$(1)-$(2).txt
	arm-none-eabi-objcopy -O ihex $(OUTDIR2)/$(2)/runner $(OUTDIR2)/$(2)/runner-$(1).ihex
	mv $(OUTDIR2)/$(2)/runner $(OUTDIR2)/$(2)/runner-$(1)
	arm-none-eabi-readelf -l $(OUTDIR2)/$(2)/runner-$(1) | grep LOAD
endef

$(eval $(call build-template,proto1,debug))
$(eval $(call build-template,proto1,release))
$(eval $(call build-template,nrfdk,debug))
$(eval $(call build-template,nrfdk,release))
$(eval $(call build-template,nk3mini,debug))
$(eval $(call build-template,nk3mini,release))

clean:
	rm -rf target

update:
	cargo update

deny:
	cargo deny -c always check 2> deny.log

program-proto1:
	nrfjprog -f NRF52 -s $(SERIAL_PROTO1) --sectorerase --program $(OUTDIR2)/release/runner-proto1.ihex
	nrfjprog -f NRF52 -s $(SERIAL_PROTO1) --reset

program-nrfdk:
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --sectorerase --program $(OUTDIR2)/release/runner-nrfdk.ihex
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --reset

reset-proto1:
	nrfjprog -f NRF52 -s $(SERIAL_PROTO1) --chiperase --program $(OUTDIR2)/release/runner-proto1.ihex
	nrfjprog -f NRF52 -s $(SERIAL_PROTO1-f NRF52 -s $(SERIAL_NRFDK) ) --program fidokeys_lfs_e0000+20000.ihex
	nrfjprog -f NRF52 -s $(SERIAL_PROTO1) --reset

reset-nrfdk:
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --chiperase --program $(OUTDIR2)/release/runner-nrfdk.ihex
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --program fidokeys_lfs_e0000+20000.ihex
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --reset

reset-nk3mini:
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --chiperase --program $(OUTDIR2)/release/runner-nk3mini.ihex
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --program fidokeys_lfs_e0000+20000.ihex
	# set APPROTECT (disable debugging port - reset with: `nrfjprog -f NRF52 --recover` or --chiperase(?)
	#nrfjprog --memwr 0x10001208 --val 0xFFFFFF00	
	# set UICR::REGOUT0 to 3v3
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --memwr 0x10001304 --val 0xfffffffc
	# set UICR::NFCPINS to disabled
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --memwr 0x1000120C --val 0xfffffffe
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --reset
	
reset-nk3mini-bootloader:
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --eraseall 
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --program mbr.hex --sectorerase
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --reset
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --program bootloader.hex --sectorerase
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --reset
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --program $(OUTDIR2)/release/runner-nk3mini.ihex
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --program fidokeys_lfs_e0000+20000.ihex
	# set APPROTECT (disable debugging port - reset with: `nrfjprog -f NRF52 --recover` or --chiperase(?)
	#nrfjprog --memwr 0x10001208 --val 0xFFFFFF00	
	# set UICR::REGOUT0 to 3v3
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --memwr 0x10001304 --val 0xfffffffc
	# set UICR::NFCPINS to disabled
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --memwr 0x1000120C --val 0xfffffffe
	nrfjprog -f NRF52 -s $(SERIAL_NRFDK) --reset


flashdump:
	rm -f flashdump_text.raw flashdump_lfs2.raw flashtmp.raw flashtmp.hex
	nrfjprog -f NRF52 --readcode flashtmp.hex
	arm-none-eabi-objcopy -I ihex -O binary --adjust-vma 0 flashtmp.hex flashtmp.raw
	dd status=none bs=4096 skip=0 count=$$(( 0xe0 )) if=flashtmp.raw of=flashdump_text.raw
	dd status=none bs=4096 skip=$$(( 0xe0 )) count=$$(( 0x20 )) if=flashtmp.raw of=flashdump_lfs2.raw
	rm -f flashtmp.raw flashtmp.hex
