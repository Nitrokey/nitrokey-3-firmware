# NRF52840 port of the Solo2 firmware

## USB Communication State

USB stack is operational.

Fido and Admin app are included. Following commands complete:

solo key ping
solo key version
solo key wink
solo key rng hexbytes
solo key set-pin
solo key credential ls

## Notes on Building

The included Makefile creates 'release' builds for both boards (NRF52840 DK
and the first prototype, called 'nrfdk' and 'proto1', respectively).

The internal flash is occupied both by the executable firmware (roughly
0x00000 - 0x49000) and the internal storage (0xe0000 - 0xfffff).

Littlefs2 mount logic feels brittle; if the board refuses to come up,
recompile the firmware with the "reformat-flash" feature flag. This
forcibly erases the internal storage area (takes quite some time during
bootup).

