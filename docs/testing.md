# Testing Guide

This guide contains information on testing beta releases of the Nitrokey 3 firmware.

## Release Candidates

Beta releases of the Nitrokey 3 firmware are published as release candidates.  You can find them in the [Releases][] section on GitHub.  Release candidates have a `-rc.<version>` suffix, for example `v1.0.3-rc.1`.  (Release candidates are tagged as pre-releases on GitHub so they are not shown in the Releases section on the repository home page.)

[Releases]: https://github.com/Nitrokey/nitrokey-3-firmware/releases

**Warning:** While we generally try to make sure that release candidates don’t contain bugs, they are not tested as well as regular releases.  If you install them, this is at your own risk.  In the worst case, installing a release candidate might destroy the user data and keys stored on your device.

## Installing Release Candidates

*Remember that you install release candidates at your own risk and might experience bugs or data loss!*

You can mostly follow the steps from the [firmware update documentation][] on docs.nitrokey.com.  A notable difference is that `nitropy nk3 update` only downloads regular releases, so you have to manually download the firmware image with `nitropy nk3 fetch-update --variant <variant> --version <version>` and then pass the path of the downloaded image to `nitrokey nk3 update`, for example:

```
$ nitropy nk3 fetch-update --variant lpc55 --version v1.0.3-rc.1
Command line tool to interact with Nitrokey devices 0.4.26
Download v1.0.3-rc.1: 100%|██████████████████████████████████████████| 305k/305k [00:00<00:00, 5.63MB/s]
Successfully downloaded firmware release v1.0.3-rc.1 to ./firmware-nk3xn-lpc55-v1.0.3-rc.1.sb2
$ nitropy nk3 update firmware-nk3xn-lpc55-v1.0.4-rc.1.sb2
```

`version` is the firmware version to download, `variant` is the hardware variant (currently `lpc55` for NK3AN and NK3CN or `nrf52` for NK3AM).  To determine the hardware variant of your device, reboot to the bootloader (`nitropy nk3 reboot --bootloader`) and then check the output of `nitropy nk3 list`:

```
$ nitropy nk3 list
Command line tool to interact with Nitrokey devices 0.4.26
:: 'Nitrokey 3' keys
/dev/hidraw4: Nitrokey 3 Bootloader (LPC55)
```

[firmware update documentation]: https://docs.nitrokey.com/nitrokey3/linux/firmware-update.html
