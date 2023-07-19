.PHONY: all
all: check lint

.PHONY: check
check:
	cargo check
	cargo check --features test
	cargo check --features provisioner

.PHONY: lint
lint:
	cargo clippy --no-deps
	cargo fmt -- --check

.PHONY: attach
attach:
	lsmod | grep vhci-hcd || sudo modprobe vhci-hcd
	sudo usbip list -r "localhost"
	sudo usbip attach -r "localhost" -b "1-1"
	sudo usbip attach -r "localhost" -b "1-1"
	sleep 5
