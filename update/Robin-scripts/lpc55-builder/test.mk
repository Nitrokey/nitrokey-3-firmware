# test.sh is the relevant script
# you’ll have to update the variables
# and you have to update the jlink-erase-reset.txt to point to the provisioner


# fully erase via debugger
# flash old v1.2.2 provisioner via debugger
# reset via debugger and boot into firmware
# provisioner firmware sets up file system
# reboot to bootloader
# flash test firmware via bootloader
# reboot into firmware mode

RUNNER=/home/sz/work/nitrokey-3-firmware/runners/embedded
BINARY=$(RUNNER)/artifacts/runner-lpc55-nk3xn.bin
# LPC55_BUILDER=/tmp/lpc55-builder
LPC55_BUILDER=$(PWD)

all: prov
	-rm "$(BINARY)"
	cd $(RUNNER) && git checkout -- Cargo.lock
	# cd $(RUNNER) && cargo update -p trussed
#	cd $(RUNNER) && cargo update
	make -C "$(RUNNER)" build-nk3xn FEATURES=alpha
	# make -C "$(RUNNER)" build-nk3xn FEATURES=opcard,trussed/clients-3

	ls -lh /tmp/provisioner-nk3xn-lpc55-v1.2.2.bin
	JLinkExe -device LPC55S69_M33_0 -if SWD -autoconnect 1 -speed 4000 -CommandFile "$(LPC55_BUILDER)"/jlink-erase-reset.txt
	./scripts/usbwait.sh 20a0:42b2

	nitropy nk3 reboot --bootloader
	./scripts/usbwait.sh 20a0:42dd 1fc9:0021

	lpc55 write-flash "$(BINARY)"
	sha256sum "$(BINARY)"
	lpc55 reboot
	./scripts/usbwait.sh 20a0:42b2

# alias for the full path
prov: /tmp/provisioner-nk3xn-lpc55-v1.2.2.bin  

/tmp/provisioner-nk3xn-lpc55-v1.2.2.bin:
	 cp ../provisioner-nk3xn-lpc55-v1.2.2.bin  /tmp/
