APPNAME=usbip-simulation
APPPATH=target/debug/usbip-simulation

all: | start-sim attach finish-message

.PHONY: finish-message
finish-message:
	@echo "###################################################"
	@echo "Done. Device should be visible in your system now. Run 'make stop' to disconnect it."

.PHONY: start-sim
start-sim: $(APPNAME)
	-$(MAKE) stop
	env RUST_LOG=debug ./$(APPNAME) &
	sleep 1

.PHONY: attach
attach: 
	lsmod | grep vhci-hcd || sudo modprobe vhci-hcd
	sudo usbip list -r "localhost"
	sudo usbip attach -r "localhost" -b "1-1"
	sudo usbip attach -r "localhost" -b "1-1"
	sleep 5

.PHONY: build
build: $(APPNAME)
	cp $(APPPATH) ./$(APPNAME) -v

.PHONY: build-clean
build-clean: | clean build

.PHONY: $(APPNAME)
$(APPNAME):
	 cargo build --features=enable-logs

.PHONY: stop
stop:
	-sudo usbip detach -p "00"
	killall $(APPNAME)

.PHONY: setup-fedora
setup-fedora:
	sudo dnf install usbip clang-libs-13.0.0
	sudo ln -s /usr/lib64/libclang.so.13 /usr/lib64/libclang.so

.PHONY: clean
clean:
	cargo clean
	rm $(APPNAME) -v

.PHONY: build-docker
CMD=make -C /app/runners/pc-usbip/ build
build-docker:
	docker build -t usbip .
	mkdir -p cargo-cache
	docker run -it --rm -v $(PWD)/cargo-cache:/root/.cargo -v $(PWD)/../../:/app usbip $(CMD)
	touch $(APPNAME)