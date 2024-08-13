# Changelog

## Unreleased

### Features

- fido-authenticator: Implement the largeBlobKey extension and the largeBlobs command ([fido-authenticator#38][])
- piv: Fix crash when changing PUK ([piv-authenticator#38][])
- OpenPGP: add support for additional curves when using the se050 backend: ([#524][])
  - NIST P-384
  - NIST P-521
  - brainpoolp256r1
  - brainpoolp384r1
  - brainpoolp512r1

[fido-authenticator#38]: https://github.com/Nitrokey/fido-authenticator/issues/38
[piv-authenticator#38]: https://github.com/Nitrokey/piv-authenticator/issues/38
[#524]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/524

## v1.7.2 (2024-06-11)

### Bugfixes

- fido-authenticator: Fix incompatibility when enumerating resident keys with libfido2/ssh-agent ([#496][])
- Ensure that an application reset erases all relevant objects on the secure element ([trussed-se050-backend#30][])

[#496]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/496
[trussed-se050-backend#30]: https://github.com/Nitrokey/trussed-se050-backend/pull/30

## v1.7.1 (2024-05-03)

### Bugfixes

- secrets-app: Require PIN for registering Reverse HOTP credentials ([trussed-secrets-app#114][])

[trussed-secrets-app#114]: https://github.com/Nitrokey/trussed-secrets-app/pull/114

## v1.7.0 (2024-04-24)

This release adds SE050 support to opcard, updates fido-authenticator to support CTAP 2.1 and introduces app and device factory reset.

### Features

- Report errors when loading the configuration during initialization and disable opcard if an error occured ([#394][])
- Fix LED during user presence check for NK3AM ([#93][])
- fido-authenticator: Implement CTAP 2.1
- OpenPGP: fix locking out after an aborted factory-reset operation ([#443][])
- Add an SE050 driver and its tests ([#335][])
- Use SE050 entropy to bootstrap the random number generator ([#335][])
- Enable SE050 support in OpenPGP by default ([#471][])
- Support app and device factory reset ([#383][], [#479][])

### Notes

- When upgrading from the test firmware release v1.6.0-test.20231218, OpenPGP keys will not be retained after the update if the `opcard.use_se050_backend` config option has been set to true.

[#93]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/93
[#335]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/335
[#383]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/383
[#394]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/394
[#443]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/443
[#471]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/471
[#479]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/479

## v1.6.0 (2023-11-23)

### Features

- usbip: Add user presence check ([#314][], [#321][])
- admin-app: Add config mechanism ([#344][])

### Changed

- secrets-app: Update to v0.13.0-rc.1
  - Confirm credential removal with a touch ([trussed-secrets-app#92][])
  - Allow to update credential ([trussed-secrets-app#65][])
- Improve stack usage of several components ([#353][])
- Reject APDU commands from multiple transports ([apdu-dispatch#19][])

### Fixed

- fido-authenticator: Reduce the maximum credential ID length for improved compatibility ([fido-authenticator#37][])
- fido-authenticator: Multiple changes to improve compliance with the specification (overview: [fido-authenticator#6][])
- Upgrade opcard to v1.2.0, fixing memory issues when using multiple RSA keys, potential data corruption, correct handling of non canonical curve25519 public keys and properly rejecting NFC requests ([#376][])
- Correct maximum binary size for LPC55 and only enable PRINCE for the subregions used for the filesystem ([#355][])
- lpc55: Move USB initialization to the end of the boot process to make sure that the device can respond to all requests, fixing a potential delay when connecting the device under Linux ([#302][])

[#302]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/302
[#376]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/376
[#314]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/314
[#321]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/321
[#335]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/335
[#344]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/344
[#353]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/353
[#355]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/355
[apdu-dispatch#19]: https://github.com/trussed-dev/apdu-dispatch/pull/19
[fido-authenticator#6]: https://github.com/Nitrokey/fido-authenticator/issues/6
[fido-authenticator#37]: https://github.com/trussed-dev/fido-authenticator/issues/37
[trussed-secrets-app#44]: https://github.com/Nitrokey/trussed-secrets-app/issues/44
[trussed-secrets-app#65]: https://github.com/Nitrokey/trussed-secrets-app/issues/65
[trussed-secrets-app#92]: https://github.com/Nitrokey/trussed-secrets-app/issues/92

## v1.5.0 (2023-05-31)

### Features

- Upgrade the secrets function to version 0.11.0, adding support for static passwords, and KeepassXC integration ([#278][])

### Changed

- Upgrade the OpenPGP function to version 1.1.0, fixing minor specification compliance issues and an unlikely data corruption scenario

### Fixed

- Upgrade ctaphid-dispatch, fixing panics after cancelled operations

[#278]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/278
[#277]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/277

## v1.4.0 (2023-05-05)

This release adds OpenPGP card support and updates the OTP functionality.

### Features

- usbip: Add `--efs` option to store the external filesystem in a file.
- Add variant to the status reported by admin-app ([#206][])
- fido-authenticator: Limit number of resident credentials to ten ([#207][])
- Add opcard to the stable firmware ([#100][])

### Changed

- Update applications:
  - opcard v1.0.0
  - piv-authenticator v0.2.0
  - secrets-app v0.10.0

[#100]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/100
[#206]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/206
[#207]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/207

## v1.3.1 (2023-04-05)

This release adds OTP functionality and contains some bugfixes.

**Warning:** On Nitrokey 3 Mini devices, this release causes a migration of the internal filesystem.  See the [Release Notes][v1.3.1] on GitHub for more information.

[v1.3.1]: https://github.com/Nitrokey/nitrokey-3-firmware/releases/tag/v1.3.1

### Features

- Add secrets app ([#186][]), implementing OTP functionality
- Return full version in status command ([#172][])
- Return storage information in status command ([#183][])
- Reduce risk of data loss by adding journaling to the internal flash ([#160][])

### Changed

- LPC55: use the embedded runner ([#97][])

### Bugfixes

- Use upstream usbd-ccid, including fixed panics and compatibility issues ([#164][])
- Improve compatibility of FIDO ([#180][])
- Fix a panic with ctaphid ([#184][])

[#186]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/186
[#184]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/184
[#183]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/183
[#180]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/180
[#172]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/172
[#164]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/164
[#160]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/160
[#97]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/97

## v1.3.0 (2023-03-27)

This release was skipped to fix a naming inconsistency.

## v1.2.2 (2022-10-05)

This release contains additional internal tests.
v1.2.1 was skipped due to an incorrectly determined bugfix.

### Bugfixes

- change fido-authenticator version from 0.1 to 0.1.1 (not needed, to be reverted) ([#87][])

### Features

- add proper `Reboot::is_locked` for nRF52 ([#89][])
- add i2c/se050 test to LPC55 (panicks in provisioner mode) ([#90][])

[#89]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/89
[#90]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/90
[#87]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/90

## v1.2.0 (2022-08-30)

This release contains various bugfixes and stability improvements.

### Bugfixes

- fido-authenticator: Return an error instead of panicking if the credential ID is too long ([#49][])
- Implement CCID abort handling, fixing an issue where GnuPG would stall for up to a minute on the first operation if a Nitrokey 3 is connected and recognized as a CCID device ([#22][])
- fido-authenticator: Fix handling of U2F commands over NFC ([fido-authenticator#18][])
- interchange: Fix unsound usage of `UnsafeCell` ([interchange#4][])
- Improve APDU handling ([iso7816#4][], [iso7816#5][], [apdu-dispatch#5][])
- Update all dependencies

[#22]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/22
[#49]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/49
[apdu-dispatch#5]: https://github.com/solokeys/apdu-dispatch/pull/5
[fido-authenticator#18]: https://github.com/solokeys/fido-authenticator/pull/18
[interchange#4]: https://github.com/trussed-dev/interchange/pull/4
[iso7816#4]: https://github.com/ycrypto/iso7816/pull/4
[iso7816#5]: https://github.com/ycrypto/iso7816/pull/5

## v1.1.0 (2022-08-02)

This release adds support for the NRF52 MCU, changes the LED color to red on
panics and allows the user to skip the additional user presence check for the
first FIDO2 operation within two seconds after boot.

### Features

- `embedded` runner to allow building for different SoCs from within a common code-base
- This pre-release only includes binaries for the nRF52 
- All features from the v1.0.4 release are included 
- Change the LED color to red on panics ([#52][])
- Skip the additional user presence check for the first Get Assertion or Authenticate request within two seconds after boot ([#61][])

[#52]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/52
[#61]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/61

## v1.0.4 (2022-07-14)

This release improves compatibility with Windows 10 and with OpenSSH and changes the LED patterns.

### Features

- Change the LED patterns so that the LED is off by default, blinks white during a user confirmation request and blinks blue when winking ([#34][])
- Add a single white LED blink for 0.5 seconds after startup ([#34][])
- Support retrieval of OpenSSH resident keys ([#48][])

### Bugfixes

- Improve stability of FIDO2 operations on Windows 10 ([#54][])

[#34]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/34
[#48]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/48
[#54]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/54

## v1.0.3 (2022-04-11)

This release fixes a FIDO authentication issue with Google.

### Bugfixes

- Correct the FIDO2 attestation certificate (fixes authentication issue with Google, [#36][])

[#36]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/36

## v1.0.2 (2022-01-26)

This release improves compatibility with Windows systems.

### Bugfixes

- usbd-ctaphid: fix ctaphid keepalive messages - fixes "busy" issue under Windows  ([#21][]) 

[#21]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/21

## v1.0.1 (2022-01-15)

This release fixes some issues with the FIDO authenticator and the admin
application.

### Features

- Change LED color and device name if provisioner app is enabled.

### Bugfixes

- fido-authenticator: use smaller CredentialID - fixes issues with some services FIDO usage ([fido-authenticator#8][])
- trussed: update P256 library - fixes signing failure in some cases ([#31][])
- admin-app: Fix CTAPHID command dispatch ([#8][]).
- admin-app: Fix CTAPHID wink command ([#9][]).
- fido-authenticator: Handle pin protocol field in hmac-secret extension data
  to fix the authenticatorGetAssertion command for newer clients ([#14][],
  [fido-authenticator#1][]).
- fido-authenticator: Signal credential protetection ([fido-authenticator#5][]).

[#8]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/8
[#9]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/9
[#14]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/14
[#31]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/31
[fido-authenticator#1]: https://github.com/solokeys/fido-authenticator/pull/1
[fido-authenticator#5]: https://github.com/solokeys/fido-authenticator/pull/5
[fido-authenticator#8]: https://github.com/solokeys/fido-authenticator/pull/8

## v1.0.0 (2021-10-16)

First stable firmware release with FIDO authenticator.
