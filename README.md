# Nitrokey 3 Firmware

This repository contains the firmware of Nitrokey 3 USB keys.

## About

The Nitrokey 3 firmware is written in [Rust][].  It uses the [Trussed][] firmware framework and is developed in collaboration with [SoloKeys][] (see the [solo2][] repository).

[Rust]: https://rust-lang.org
[Trussed]: https://trussed.dev/
[SoloKeys]: https://solokeys.com/
[solo2]: https://github.com/solokeys/solo2

## Documentation

Documentation for users is available in the [Nitrokey 3 section on docs.nitrokey.com][docs.nitrokey.com].  For developer documentation, see the [`docs`](./docs/index.md) directory.

[docs.nitrokey.com]: https://docs.nitrokey.com/nitrokey3/index.html

## Dependencies

To build the firmware from source, you need these dependencies:

- Rust (current stable release for the `thumbv8m.main-none-eabi` target with the `llvm-tools-preview` component)
- clang with development headers
- [`flip-link`][]
- [`cargo-binutils`][]

[`flip-link`]: https://github.com/knurling-rs/flip-link
[`cargo-binutils`]: https://github.com/rust-embedded/cargo-binutils

To flash the firmware to the device, you need [`mboot`][] or [`lpc55`][].

[`mboot`]: https://github.com/molejar/pyMBoot
[`lpc55`]: https://github.com/lpc55/lpc55-host

## License

This software is fully open source.

All software, unless otherwise noted, is dual licensed under [Apache 2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT).
You may use the software under the terms of either the Apache 2.0 license or MIT license.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Funding

[<img src="https://nlnet.nl/logo/banner.svg" width="200" alt="Logo NLnet: abstract logo of four people seen from above" hspace="20">](https://nlnet.nl/)
[<img src="https://nlnet.nl/image/logos/NGI0PET_tag.svg" width="200" alt="Logo NGI Zero: letterlogo shaped like a tag" hspace="20">](https://nlnet.nl/NGI0/)

This project was funded through the [NGI0 PET](https://nlnet.nl/PET) Fund, a fund established by [NLnet](https://nlnet.nl/) with financial support from the European Commission's [Next Generation Internet programme](https://ngi.eu/), under the aegis of DG Communications Networks, Content and Technology under grant agreement No 825310.
