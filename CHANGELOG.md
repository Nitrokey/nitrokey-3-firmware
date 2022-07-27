# Unreleased

- include the embedded runner, currently only confirmed working for nRF52
- Change the LED color to red on panics ([#52][])
- Skip the additional user presence check for the first Get Assertion or Authenticate request within two seconds after boot ([#61][])

[#52]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/52
[#61]: https://github.com/Nitrokey/nitrokey-3-firmware/issues/61

# v1.1.0-rc.1 (2022-07-27)

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
