all: build

.PHONY: build run connect clean generate_all kill test

#inspired by https://troubles.noblogs.org/post/2020/06/26/openssh-keys-on-a-fido2-dongle/

KEY=data/my_ecdsa_sk
ALL_KEYS=data/my_ecdsa_sk data/ed25519-sk data/my_ecdsa_sk-resident data/ed25519-sk-resident

generate_all: | $(ALL_KEYS)

build: generate_all
	cat data/*.pub > key.pub
	sudo docker build -t sshd-sk:0.1 .


test:
	$(MAKE) run &
	sleep 2
	$(MAKE) connect KEY=data/my_ecdsa_sk-resident
	$(MAKE) connect KEY=data/my_ecdsa_sk
	$(MAKE) connect KEY=data/ed25519-sk-resident
	$(MAKE) connect KEY=data/ed25519-sk
	$(MAKE) kill

CONT_NAME=SSH_TEST

run:
	sudo docker run --rm --name $(CONT_NAME) -p 10022:22 sshd-sk:0.1 | ts -i

kill:
	sudo docker kill $(CONT_NAME)


connect: $(KEY)
	ssh -i $<  -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no uzer@localhost -p 10022 cat /works
	@echo "Tested key: $(KEY)"
	@echo ""

data/my_ecdsa_sk:
	ssh-keygen -t ecdsa-sk -f $@

data/ed25519-sk:
	ssh-keygen -t ed25519-sk -f $@

data/ed25519-sk-resident:
	ssh-keygen -t ed25519-sk -Oresident -Oapplication=ssh:foobar -f $@

data/my_ecdsa_sk-resident:
	ssh-keygen -t ecdsa-sk -Oresident -Oapplication=ssh:foobar2 -f $@

clean:
	-rm -v data/* key.pub
	-sudo docker rmi sshd-sk:0.1 -f