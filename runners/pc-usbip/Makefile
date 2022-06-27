APPNAME=usbip-simulation
APPPATH=target/debug/usbip-simulation

all: | start-sim attach finish-message

.PHONY: finish-message
finish-message:
	@echo "###################################################"
	@echo "Done. Device should be visible in your system now. Run 'make stop' to disconnect it."

.PHONY: start-sim
start-sim:
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
	cargo build

.PHONY: stop
stop:
	-sudo usbip detach -p "00"
	killall $(APPNAME)

.PHONY: setup-fedora
setup-fedora:
	sudo dnf install usbip

.PHONY: clean
clean:
	cargo clean
	rm $(APPNAME) -v

.PHONY: build-docker
build-docker:
	docker build -t usbip .
	mkdir -p cargo-cache
	docker run -it --rm -v $(PWD)/cargo-cache:/root/.cargo -v $(PWD)/../../:/app usbip make -C /app/runners/pc-usbip/ build
	touch $(APPNAME)