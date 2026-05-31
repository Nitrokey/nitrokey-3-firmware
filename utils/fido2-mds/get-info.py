import json

from fido2.ctap2.base import Ctap2
from fido2.hid import CtapHidDevice


FIELDS = {
    0x01: "versions",
    0x02: "extensions",
    0x03: "aaguid",
    0x04: "options",
    0x05: "maxMsgSize",
    0x06: "pinUvAuthProtocols",
    0x07: "maxCredentialCountInList",
    0x08: "maxCredentialIdLength",
    0x09: "transports",
    0x0A: "algorithms",
    0x0B: "maxSerializedLargeBlobArray",
    0x0C: "forcePINChange",
    0x0D: "minPINLength",
    0x0E: "firmwareVersion",
    0x0F: "maxCredBlobLength",
    0x10: "maxRPIDsForSetMinPINLength",
    0x11: "preferredPlatformUvAttempts",
    0x12: "uvModality",
    0x13: "certifications",
    0x14: "remainingDiscoverableCredentials",
    0x15: "vendorPrototypeConfigCommands",
    0x16: "attestationFormats",
    0x17: "uvCountSinceLastPinEntry",
    0x18: "longTouchForReset",
    0x19: "encIdentifier",
    0x1A: "transportsForReset",
    0x1B: "pinComplexityPolicy",
    0x1C: "pinComplexityPolicyURL",
    0x1D: "maxPINLength",
    0x1E: "encCredStoreState",
    0x1F: "authenticatorConfigCommands",
}


if __name__ == "__main__":
    devices = list(CtapHidDevice.list_devices())
    assert len(devices) == 1
    device = devices[0]

    ctap2 = Ctap2(device)
    raw_info = ctap2.send_cbor(Ctap2.CMD.GET_INFO)

    info = {}
    for i in raw_info:
        field = FIELDS[i]
        value = raw_info[i]
        if field == "aaguid":
            value = value.hex()
        info[field] = value

    print(json.dumps(info, indent=4))
