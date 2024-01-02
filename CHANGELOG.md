# Unreleased

### Features

- Add an SE050 driver and its tests ([#335][])
- Use SE050 entropy to bootstrap the random number generator ([#335][])
- fido-authenticator: Implement the largeBlobKey extension and the largeBlobs command ([fido-authenticator#38][])
- Report errors when loading the configuration during initialization and disable opcard if an error occured ([#394][])
- piv: Fix crash when changing PUK ([piv-authenticator#38][])

[#394]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/394
[fido-authenticator#38]: https://github.com/Nitrokey/fido-authenticator/issues/38
[piv-authenticator#38]: https://github.com/Nitrokey/piv-authenticator/issues/38

# 1.6.0 (2023-11-23)

- no additions since v1.6.0-rc.1

# 1.6.0-rc.1 (2023-11-10)

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


# v1.5.0 (2023-05-31)

### Features

- Upgrade the secrets function to version 0.11.0, adding support for static passwords, and KeepassXC integration ([#278][])

### Changed

- Upgrade the OpenPGP function to version 1.1.0, fixing minor specification compliance issues and an unlikely data corruption scenario

### Fixed

- Upgrade ctaphid-dispatch, fixing panics after cancelled operations

[#278]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/278
[#277]: https://github.com/Nitrokey/nitrokey-3-firmware/pull/277

# v1.4.0 (2023-05-05)

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

# v1.3.1 (2023-04-05)

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

# v1.3.0 (2023-03-27)

This release was skipped to fix a naming inconsistency.

# v1.2.2 (2022-10-05)

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

# v1.2.0 (2022-08-30)

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

# v1.1.0 (2022-08-02)

This release adds support for the NRF52 MCU, changes the LED color to red on
panics and allows the user to skip the additional user presence check for the
first FIDO2 operation within two seconds after boot.

## v1.1.0-rc.1 (2022-07-27)

This is the first official nRF52 release(candidate) for the Nitrokey 3A Mini.

### Features

- `embedded` runner to allow building for different SoCs from within a common code-base
- This pre-release only includes binaries for the nRF52 
- All features from the v1.0.4 release are included 
- Change the LED color to red on panics ([#52][])
- Skip the additional user presence check for the first Get Assertion or Authenticate request within two seconds after boot ([#61][])

[#52]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/52
[#61]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/61

# v1.0.4 (2022-07-14)

This release improves compatibility with Windows 10 and with OpenSSH and changes the LED patterns.

### Features

- Use white instead of blue as the LED color for winking ([#34][])

## v1.0.4-rc.3 (2022-07-05)

### Features

- Change the LED patterns so that the LED is off by default, blinks white during a user confirmation request and blinks blue when winking ([#34][])
- Add a single white LED blink for 0.5 seconds after startup ([#34][])
- Support retrieval of OpenSSH resident keys ([#48][])

### Bugfixes

- Improve stability of FIDO2 operations on Windows 10 ([#54][])

[#34]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/34
[#48]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/48
[#54]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/54

# v1.0.3 (2022-04-11)

This release fixes a FIDO authentication issue with Google.

## v1.0.3-rc.1 (2022-04-06)

### Bugfixes

- Correct the FIDO2 attestation certificate (fixes authentication issue with Google, [#36][])

[#36]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/36

# v1.0.2 (2022-01-26)

This release improves compatibility with Windows systems.

## v1.0.2-rc.1 (2022-01-25)

Update to upstream release 1.0.9.

### Bugfixes

- usbd-ctaphid: fix ctaphid keepalive messages - fixes "busy" issue under Windows  ([#21][]) 

[#21]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/21

# v1.0.1 (2022-01-15)

This release fixes some issues with the FIDO authenticator and the admin
application.

### Bugfixes

- fido-authenticator: use smaller CredentialID - fixes issues with some services FIDO usage ([fido-authenticator#8][])
- trussed: update P256 library - fixes signing failure in some cases ([#31][])

[#31]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/31
[fido-authenticator#8]: https://github.com/solokeys/fido-authenticator/pull/8

## v1.0.1-rc.1 (2021-12-06)

### Features

- Change LED color and device name if provisioner app is enabled.

### Bugfixes

- admin-app: Fix CTAPHID command dispatch ([#8][]).
- admin-app: Fix CTAPHID wink command ([#9][]).
- fido-authenticator: Handle pin protocol field in hmac-secret extension data
  to fix the authenticatorGetAssertion command for newer clients ([#14][],
  [fido-authenticator#1][]).
- fido-authenticator: Signal credential protetection ([fido-authenticator#5][]).

[#8]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/8
[#9]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/9
[#14]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/14
[fido-authenticator#1]: https://github.com/solokeys/fido-authenticator/pull/1
[fido-authenticator#5]: https://github.com/solokeys/fido-authenticator/pull/5

# v1.0.0 (2021-10-16)

First stable firmware release with FIDO authenticator.
