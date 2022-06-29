#!/bin/bash

set -euxo pipefail

mkdir nrf-deploy

git clone git@git.nitrokey.com:robin/test-certificates.git nrf-deploy/test-certs

git clone git@github.com:Nitrokey/nitrokey-3-firmware.git nrf-deploy/fw-repo

pushd nrf-deploy/fw-repo
git checkout embedded-runner-3
popd

cp -r nrf-deploy/test-certs/nk3/firmware-nrf52 nrf-deploy/fw-repo/runners/embedded/nrf-bootloader/signing-key
cp nrf-deploy/fw-repo/runners/embedded/nrf-bootloader/signing-key/dfu_private.{pem,key}

pushd nrf-deploy/fw-repo/runners/embedded/
make build-bootloader
cp nrf-bootloader/mbr.hex ../../../
cp nrf-bootloader/bootloader.hex ../../../

make build-nk3am.bl FEATURES=provisioner
cp artifacts/runner-nrf52-bootloader-nk3am.bin.ihex fw_unsigned.hex
bash nrf-bootloader/sign.sh 15 fw_signed_update.zip fw_unsigned.hex nrf-bootloader/signing-key/dfu_private.key
cp fw_signed_update.zip ../../../provisioner-firmware.zip
make clean-nk3am.bl

make build-nk3am.bl FEATURES=develop
cp artifacts/runner-nrf52-bootloader-nk3am.bin.ihex fw_unsigned.hex
bash nrf-bootloader/sign.sh 15 fw_signed_update.zip fw_unsigned.hex nrf-bootloader/signing-key/dfu_private.key
cp fw_signed_update.zip ../../../develop-firmware.zip
make clean-nk3am.bl

popd

pushd nrf-deploy
ls --size -1 bootloader.hex mbr.hex develop-firmware.zip provisioner-firmware.zip

