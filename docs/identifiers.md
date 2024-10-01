# Nitrokey 3 Identifiers

This document lists identifiers used by Nitrokey 3 devices.

## Part Numbers and Project Names

| Device        | Part Number | Project Name | USB    | NFC | SoC   |
| ------------- | ----------- | ------------ | :----: | :-: | :---: |
| Nitrokey 3 AM | NK3AM1xx    | Athene       | Type A | no  | nrf52 |
| Nitrokey 3 AN | NK3AN1xx    | Gaia         | Type A | yes | lpc55 |
| Nitrokey 3 CN | NK3CN1xx    | Hades        | Type C | yes | lpc55 |


## USB Vendor and Product ID

| Device                         | Vendor ID | Product ID |
| ------------------------------ | --------: | ---------: |
| Nitrokey 3                     | 0x20a0    | 0x42b2     |
| Nitrokey 3 Bootloader (lpc55)  | 0x20a0    | 0x42dd     |
| Nitrokey 3 Bootloader (nrf52)  | 0x20a0    | 0x42e8     |
| Nitrokey Passkey               | 0x20a0    | 0x42f3     |
| Nitrokey Passkey Bootloader    | 0x20a0    | 0x42f4     |

## FIDO2 AAGUID

| Device                | AAGUID                                 |
| --------------------- | -------------------------------------- |
| Nitrokey 3 xN (lpc55) | `ec99db19-cd1f-4c06-a2a9-940f17a6a30b` |
| Nitrokey 3 AM (nrf52) | `2cd2f727-f6ca-44da-8f48-5c2e5da000a2` |
| Nitrokey Passkey      | `9a03e537-4cbe-4a01-b2e2-242e0dd9a59b` |

## FIDO2 Attestation Certificate

| Device                | Firmware Version | Hash of the Attestation Certificate                                |
| --------------------- | :--------------: | ------------------------------------------------------------------ |
| Nitrokey 3 xN (lpc55) | < 1.0.3          | `ad8fd1d16f59104b9e06ef323cc03f777ed5303cd421a101c9cb00bb3fdf722d` |
| Nitrokey 3 xN (lpc55) | >= 1.0.3         | `aa1cb760c2879530e7d7fed3da75345d25774be9cfdbbcbd36fdee767025f34b` |
| Nitrokey 3 AM (nrf52) | all              | `4c331d7af869fd1d8217198b917a33d1fa503e9778da7638504a64a438661ae0` |
| Nitrokey Passkey      | all              | `c7512dfcd15ffc5a7b4000e4898e5956ee858027794c5086cc137a02cd15d123` |
| Development devices   | all              | `c7d87cac86b69059bbff5c43872a20892267518614dfc9822c7ee55ad89f0022` |

The hash is calculated as the SHA-256 digest of the FIDO2 attestation certificate in the DER format.
