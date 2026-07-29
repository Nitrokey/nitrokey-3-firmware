# Running the firmware on a SoloKeys Solo2

This document describes the `board-solo2` target, which builds this firmware for
the [SoloKeys Solo2][solo2].  It is **experimental** and **not an official
Nitrokey product configuration**.

[solo2]: https://github.com/solokeys/solo2

## Why this works

The Solo2 and the LPC55 variant of the Nitrokey 3 (`nk3xn`) are built around the
**same NXP LPC55S69 microcontroller**, and this firmware descends from the same
[`solokeys/solo2`][solo2] code base.  The two boards differ in only two places:

| Subsystem      | Nitrokey 3 (`nk3xn`)              | Solo2 (`board-solo2`)                    |
| -------------- | -------------------------------- | ---------------------------------------- |
| Buttons        | single GPIO (`Pio0_31`)          | three capacitive touch pads (ADC + DMA)  |
| Secure element | SE050 (I2C5)                     | **none**                                 |
| RGB LED        | `Pio0_5` / `Pio1_21` / `Pio1_19` | identical                                |
| NFC            | FM11NC08 (SPI, CS `Pio1_20`)     | identical                                |
| External flash | SPI (`Pio0_28/24/25`, CS `Pio0_13`) | identical                             |

Therefore `board-solo2` reuses the entire `nk3xn` board implementation and only:

* selects the capacitive touch button driver
  ([`boards::nk3xn::button_touch`](../components/boards/src/nk3xn/button_touch.rs),
  ported from the upstream `solo2` board), and
* disables the `se050` feature (there is no secure element to talk to).

The touch buttons use `Ctimer1` (charge/sample trigger), `Ctimer2` (sample
correlation), the ADC and one DMA channel.  On the Nitrokey 3 `Ctimer2` drives
the SE050 delay timer; on the Solo2 it is free because there is no SE050.

## Identity

The device keeps the **Nitrokey 3 USB identity** (VID `0x20A0`, PID `0x42B2`),
so it is managed with `nitropy nk3` and the standard tooling, not the SoloKeys
`solo2` CLI.

## Building

```
$ rustup target add thumbv8m.main-none-eabi
$ make -C runners/embedded build-solo2
```

The artifacts land in `runners/embedded/artifacts/runner-lpc55-solo2.{elf,bin}`.

A development build (no encrypted storage / secure boot, useful for first
bring-up) is:

```
$ make -C runners/embedded build-solo2 FEATURES=develop
```

## Flashing (development)

Flashing unsigned firmware requires the LPC55 ROM bootloader (ISP mode) and a
device whose **CMPA is not sealed**.

> **Warning — sealed devices.** A production Solo2 provisioned with secure boot
> and a sealed CMPA will refuse foreign firmware, and CMPA sealing is
> irreversible.  Only devices with an open/unsealed CMPA (e.g. a Solo2 "Hacker"
> or a self-provisioned unit) can be reflashed.

> **Secure boot must be disabled (or you must self-sign).** A Solo2 ships with
> `secure-boot-enabled` set to the SoloKeys root of trust, and it *is* enforced
> even when the CMPA is unsealed: an unsigned build like this one will not boot.
> On a Hacker (unsealed) you can turn it off — write a CMPA with
> `secure-boot-enabled: false` via `lpc55 configure factory-settings` — flash,
> and turn it back on later; this is reversible while `seal` stays `false`.
> To keep secure boot *on* with this firmware you must run your own root of
> trust: generate your keys, sign the image (NXP `spsdk`/`nxpimage`), and write
> your ROTKH into the CMPA — optionally keeping the SoloKeys root in a spare RoT
> slot so official releases still boot.  The LPC55 firmware version is also an
> anti-rollback monotonic counter, so a signed image must declare a version at
> least as high as the one currently recorded.

1. Put the Solo2 into LPC55 ROM bootloader mode (`solo2 program bootloader`, or
   the physical bootloader-pin method while plugging in).
2. Confirm it is visible: `lpc55 ls`.
3. Flash the firmware you built. Write the artifact directly:

   ```
   $ lpc55 write-flash runners/embedded/artifacts/runner-lpc55-solo2.bin
   $ lpc55 reboot
   ```

   > **Do not use `make -C utils/lpc55-builder bl-flash` to flash a build with
   > uncommitted changes.** That target has a `build:` prerequisite that
   > recompiles the firmware from the committed tree first, silently
   > overwriting your `.bin`. Flash the artifact directly with `lpc55
   > write-flash` (or commit your changes first).

## Production provisioning (secure boot + encrypted storage + attestation)

This mirrors the Nitrokey 3 flow in
[`docs/lpc55-quickstart.md`](./lpc55-quickstart.md) and
[`utils/lpc55-builder`](../utils/lpc55-builder); nothing here is Solo2-specific
except the firmware binary/features you build.  These steps require the physical
device, `lpc55` + `nitropy`, and your own signing material; the CMPA seal is
**not reversible**.

1. **Reset / open the device** and apply the development CMPA:
   `make -C utils/lpc55-builder reset`.
2. **Provision the keystore** (PRINCE region-2 key for encrypted internal
   storage) using a provisioner build:
   `make -C utils/lpc55-builder bl-provision-keystore`.
3. **Apply the CMPA** (`bl-config-cmpa-develop`, and for a true release build a
   secure-boot CMPA signed with your ROT keys — see the `# TODO: add secure
   boot` marker in the builder Makefile, which is not yet implemented upstream).
4. **Flash a provisioner firmware** and **provision the FIDO attestation key +
   certificate** and the Trussed device key/cert
   (`make -C utils/lpc55-builder fw-provision-certs`).  You need access to real
   FIDO2 batch keys/certs for attestation to validate; without them
   `nitropy nk3 test` reports an expected FIDO cert-hash mismatch.
5. **Flash the final firmware**:
   `make -C utils/lpc55-builder flash OUTPUT_BIN=.../runner-lpc55-solo2.bin`.
6. **Seal the CMPA** to enforce secure boot (irreversible) — only once every
   previous step is verified.

Build the provisioner/final firmware for the Solo2 by passing the board target,
e.g.:

```
$ make -C runners/embedded build-solo2 FEATURES=provisioner
```

## Status and caveats

* **Verified on real Solo2 hardware** (a "Solo 2 Security Key" Hacker).  Both the
  default **PRINCE-encrypted** build and the `develop` build boot with
  `init_status 0` and the internal and external filesystems available; the
  encrypted build is the correct one for a factory PRINCE-provisioned key (see
  the storage note below).  Confirmed working end-to-end:
  * USB enumerates as a Nitrokey 3; the **capacitive touch button** registers
    user presence in real use.
  * **FIDO2** advertises the full feature set (FIDO 2.0/2.1, `es256` + `eddsa`,
    `hmac-secret`, resident keys, USB + NFC transports).
  * **OATH/TOTP** works: an added credential survives a replug and generates
    codes that match an independent TOTP implementation bit-for-bit.
  * SE050 is correctly skipped; the only `nitropy nk3 test` failure is FIDO2
    attestation (`x5c`), which needs provisioning — see below.
  * The capacitive-touch thresholds (`[12_000, 12_000, 12_000]`, confidence
    `5`), pin assignment and channel mapping are copied verbatim from the
    official `solokeys/solo2` firmware.
  * The touch clock requires the CPU to run at ≥96 MHz; this holds in active
    (USB) mode, where the buttons are initialised.  In passive NFC mode the
    buttons are not built (the ADC drives the clock controller), exactly as on
    the Nitrokey 3.
* **Storage mode: match the build to the key.**  A Solo2 that ships
  PRINCE-provisioned needs the default **encrypted** build (`make build-solo2`,
  `require_prince = true`).  The `develop` build adds `no-encrypted-storage`
  (`require_prince = false`) and is for quick bring-up only — flashing the wrong
  mode for the key's keystore is unsupported.  The encrypted build was verified
  on a provisioned key: the internal filesystem is PRINCE-encrypted and the
  `key_provisioned(PrinceRegion2)` assertion passes.
* **External flash.**  The Solo2 ships Winbond W25Q16JV parts (JEDEC
  manufacturer `0xEF`) rather than the GigaDevice GD25Q16 (`0xC8`) used by the
  Nitrokey 3; both are accepted (see `components/boards/src/flash.rs`).
* **FIDO2 requires provisioning.**  On an unprovisioned development build the
  FIDO2 test fails with an `x5c` / certificate error because no attestation key
  and certificate have been written.  This is expected; provision them via the
  production flow above with real batch keys.
* **No SE050 means no secure element and no SE050-only mechanisms.**  The
  curves normally provided by the SE050 backend — P384, P521, the Brainpool
  P256/P384/P512 family and Secp256k1 — are unavailable and return an error at
  runtime (the same situation as the `usbip` runner).  The common mechanisms
  used by FIDO2, PIV and OpenPGP (P256, Ed25519, X25519, RSA via
  `backend-rsa`, AES, ChaCha8Poly1305, HMAC-SHA256, SHA256) are implemented by
  Trussed itself and are unaffected.  Secrets that would normally be wrapped by
  the SE050 are instead protected only by the PRINCE-encrypted internal flash.
