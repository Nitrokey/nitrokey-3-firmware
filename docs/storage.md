# Storage Overview

The Nitrokey 3 has three storage types: internal, external and volatile.  The internal storage uses the on-chip flash memory.  The external storage uses a separate flash chip.  The volatile storage uses RAM.

## Storage Sizes

| Storage          | Size   |
| ---------------- | -----: |
| Internal (lpc55) | 42 KiB |
| Internal (nrf52) | 80 KiB |
| External         | 2 MiB  |

## Usage

This section describes how the storage is used in the current stable firmware.

### Trussed

Trussed stores the RNG state on the internal filesystem (see `ServiceResources::rng`).  During provisioning, a Trussed device key and certificate are also generated on the internal filesystem.

### fido-authenticator

fido-authenticator stores its state, a KEK and the resident keys on the internal filesystem.  During provisioning, the FIDO2 attestation key and certificate are stored on the internal filesystem.  The KEK is generated on first use.  If there is not enough free space to generate the KEK, the application cannot be used.
