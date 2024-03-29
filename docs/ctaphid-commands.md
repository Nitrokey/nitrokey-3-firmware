# CTAPHID Commands

This document provides an overview of the [CTAPHID vendor commands][vendor] used by the Nitrokey 3.

| Command | Application             |
| ------: | ----------------------- |
| 0x51    | [admin-app][] (update)  |
| 0x53    | [admin-app][] (reboot)  |
| 0x60    | [admin-app][] (rng)     |
| 0x61    | [admin-app][] (version) |
| 0x62    | [admin-app][] (uuid)    |
| 0x63    | [admin-app][] (locked)  |
| 0x70    | [secrets-app][]         |
| 0x71    | [provisioner-app][]     |
| 0x72    | [admin-app][]           |

[vendor]: https://fidoalliance.org/specs/fido-v2.1-ps-20210615/fido-client-to-authenticator-protocol-v2.1-ps-20210615.html#usb-vendor-specific-commands
[admin-app]: https://github.com/Nitrokey/admin-app
[provisioner-app]: https://github.com/Nitrokey/nitrokey-3-firmware/tree/main/components/provisioner-app
[secrets-app]: https://github.com/Nitrokey/trussed-secrets-app
