# Changelog

## v6

- Update for Nitrokey 3 firmware release v2.0.0:
  - Set `authenticatorVersion` to 2.
  - Add CTAP 2.3 support.
  - Update `authenticatorGetInfo`.
- Use the FIDO intermediate CA in `attestationRootCertificates` for NK3AM test, NK3xN and NK3xN test.
  (NK3AM is already using the intermediate CA since v3.)

## v5

- Update `icon`.

## v4

- Add `cryptoStrength`.

## v3

- Use the FIDO intermediate CA in `attestationRootCertificates` for NK3AM instead of the FIDO batch certificate.
  This improves compatibility with non-complying server implementations.

## v2

- Use the FIDO batch certificate in `attestationRootCertificates` for NK3AM.
  Previously, we mistakenly used the FIDO intermediate CA for NK3xN.

## v1

Initial metadata for NK3AM and NK3xN following the [FIDO Metadata Statement specification v3.0](https://fidoalliance.org/specs/mds/fido-metadata-statement-v3.0-ps-20210518.html).
