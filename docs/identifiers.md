# Nitrokey 3 Identifiers

This document lists identifiers used by Nitrokey 3 devices.

## USB Vendor and Product ID

| Device                        | Vendor ID | Product ID |
| ----------------------------- | --------: | ---------: |
| Nitrokey 3                    | 0x20a0    | 0x42b2     |
| Nitrokey 3 Bootloader (lpc55) | 0x20a0    | 0x42dd     |
| Nitrokey 3 Bootloader (nrf52) | 0x20a0    | 0x42e8     |

## FIDO2 AAGUID

| Device                | AAGUID                                 |
| --------------------- | -------------------------------------- |
| Nitrokey 3 xN (lpc55) | `ec99db19-cd1f-4c06-a2a9-940f17a6a30b` |
| Nitrokey 3 AM (nrf52) | `2cd2f727-f6ca-44da-8f48-5c2e5da000a2` |

## FIDO2 Attestation Certificate

| Device                | Firmware Version | Hash of the Attestation Certificate                                |
| --------------------- | :--------------: | ------------------------------------------------------------------ |
| Nitrokey 3 xN (lpc55) | < 1.0.3          | `ad8fd1d16f59104b9e06ef323cc03f777ed5303cd421a101c9cb00bb3fdf722d` |
| Nitrokey 3 xN (lpc55) | >= 1.0.3         | `aa1cb760c2879530e7d7fed3da75345d25774be9cfdbbcbd36fdee767025f34b` |
| Nitrokey 3 AM (nrf52) | all              | `4c331d7af869fd1d8217198b917a33d1fa503e9778da7638504a64a438661ae0` |

The hash is calculated as the SHA-256 digest of the FIDO2 attestation certificate in the DER format.
