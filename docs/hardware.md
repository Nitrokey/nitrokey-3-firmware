# Nitrokey 3 Hardware

The Nitrokey 3 firmware is developed for this hardware:

| Component        | Devices      | Manufacturer | Part Number   | Resources                            |
| ---------------- | ------------ | ------------ | ------------- | ------------------------------------ |
| MCU              | NK3AM        | Nordic Semi  | NRF52840      | [Product Specification][nrf52840-ps] |
|                  | NK3xN        | NXP          | LPC55S69      | [Data Sheet][lpc55s69-ds]            |
| External flash   | all          | GigaDevice   | GD25Q16C      | [Data Sheet][gd25q16c-ds]            |
| Secure element   | all          | NXP          | SE050C1       | [Data Sheet][se050-ds]               |
| NFC chip         | NK3xN        | Fudan Micro  | FM11NC08      | [Data Sheet][fm11nc08-ds]            |
| RGB LED          | NK3CN        | Everlight    | EAST1616RGBB4 | [Data Sheet][east1616rgbb4-ds]       |
|                  | NK3AM, NK3AN | Würth        | 150066M153000 | [Data Sheet][150066m153000-ds]       |
| Proximity sensor | NK3xN        | Microchip    | MTCH101       | [Data Sheet][mtch101-ds]             |

[nrf52840-ps]: https://infocenter.nordicsemi.com/pdf/nRF52840_PS_v1.7.pdf
[lpc55s69-ds]: https://www.nxp.com/docs/en/nxp/data-sheets/LPC55S6x_DS.pdf
[gd25q16c-ds]: https://www.elm-tech.com/en/products/spi-flash-memory/gd25q16/gd25q16.pdf
[se050-ds]: https://www.nxp.com/docs/en/data-sheet/SE050-DATASHEET.pdf
[fm11nc08-ds]: https://eng.fmsh.com/AjaxFile/DownLoadFile.aspx?FilePath=/UpLoadFile/20140904/FM11NC08_ps_eng.pdf&fileExt=file
[east1616rgbb4-ds]: https://everlightamericas.com/index.php?controller=attachment&id_attachment=2827
[150066m153000-ds]: https://www.we-online.com/components/products/datasheet/150066M153000.pdf
[mtch101-ds]: https://ww1.microchip.com/downloads/en/DeviceDoc/40001664B.pdf

For more information, see the hardware repositories:
- [nitrokey-3a-mini-nrf52-hardware](https://github.com/Nitrokey/nitrokey-3a-mini-nrf52-hardware)
- [nitrokey-3a-nfc-lpc55-hardware](https://github.com/Nitrokey/nitrokey-3a-nfc-lpc55-hardware)
- [nitrokey-3c-nfc-lpc55-hardware](https://github.com/Nitrokey/nitrokey-3c-nfc-lpc55-hardware)

We use these debuggers for development:
- [LPC-Link2][] (LPC55)
- [LPCXpresso55S69][] (LPC55)
- [NRF52840 DK][] (NRF52)

[LPC-Link2]: https://www.embeddedartists.com/products/lpc-link2/
[LPCXpresso55S69]: https://www.nxp.com/design/software/development-software/mcuxpresso-software-and-tools-/lpcxpresso-boards/lpcxpresso55s69-development-board:LPC55S69-EVK
[NRF52840 DK]: https://www.nordicsemi.com/Products/Development-hardware/nrf52840-dk
