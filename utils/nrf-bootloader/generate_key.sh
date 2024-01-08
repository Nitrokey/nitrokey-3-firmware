#!/usr/bin/env bash

set -euxo pipefail


if [[ "$1" = "" ]]; then
	echo "Usage: $0 <target-directory>"
fi

target=$1


[ -d "$target" ] || mkdir -p $target


privkey="${target}/dfu_private.key"
pubkey="${target}/dfu_public_key.c"
pubkey_pem="${target}/dfu_public_key.pem"

if [ -r "${privkey}" ] | [ -r "${pubkey}" ]; then

	echo "either ${privkey} or ${pubkey} exists, ...."
	echo "won't overwrite them, exiting .... .."
	exit 1

fi

# Generate a private key
nrfutil keys generate ${privkey}

# Generate C source file containing the public key
nrfutil keys display --key pk --format code ${privkey} --out_file ${pubkey}
nrfutil keys display --key pk --format pem ${privkey} --out_file ${pubkey_pem}

stat ${pubkey}
stat ${privkey}

