import base64
import enum
import json
from argparse import ArgumentParser
from dataclasses import dataclass
from enum import Enum

from cairosvg import svg2png
from fido2.mds3 import MetadataStatement, VerificationMethodDescriptor, Version
from fido2.webauthn import Aaguid

UPV_CTAP_2_0 = Version(1, 0)
UPV_CTAP_2_1 = Version(1, 1)

UVD_NONE = VerificationMethodDescriptor(user_verification_method="none")
UVD_PRESENCE_INTERNAL = VerificationMethodDescriptor(
    user_verification_method="presence_internal"
)
UVD_PASSCODE_EXTERNAL = VerificationMethodDescriptor(
    user_verification_method="passcode_external"
)


@enum.unique
class CtapVersion(Enum):
    CTAP_2_0 = "2.0"
    CTAP_2_1 = "2.1"

    @classmethod
    def from_str(cls, s: str) -> "CtapVersion":
        for variant in cls:
            if variant.value == s:
                return variant
        raise ValueError("Unknown CTAP version {s}")


# see https://fidoalliance.org/specs/mds/fido-metadata-statement-v3.0-ps-20210518.html
@dataclass
class Authenticator:
    name: str
    aaguid: str
    has_nfc: bool
    attestation_root_certificate: str
    is_test: bool = False

    @property
    def attachment_hint(self) -> list[str]:
        hints = ["wired", "external"]
        if self.has_nfc:
            hints.append("nfc")
            hints.append("wireless")
        return sorted(hints)

    @property
    def transports(self) -> list[str]:
        transports = ["usb"]
        if self.has_nfc:
            transports.append("nfc")
        return sorted(transports)

    def mds(self, ctap: CtapVersion) -> MetadataStatement:
        with open(self.attestation_root_certificate, "rb") as f:
            attestation_root_certificate = f.read()

        with open("nitrokey.svg", "rb") as f:
            icon_bytes = svg2png(file_obj=f, output_width=128, output_height=128)
            icon_base64 = base64.b64encode(icon_bytes).decode()
            icon = f"data:image/png;base64,{icon_base64}"

        aaguid = Aaguid.parse(self.aaguid)

        upv = [UPV_CTAP_2_0]
        authenticator_versions = ["U2F_V2", "FIDO_2_0"]
        if ctap == CtapVersion.CTAP_2_1:
            upv.append(UPV_CTAP_2_1)
            authenticator_versions.append("FIDO_2_1")
        else:
            authenticator_versions.append("FIDO_2_1_PRE")

        options = {
            "rk": True,
            "up": True,
            "plat": False,
            "clientPin": False,
            "credMgmt": True,
            "largeBlobs": False,
        }
        if ctap == CtapVersion.CTAP_2_1:
            options["pinUvAuthToken"] = True
            pin_protocols = [2, 1]
        else:
            options["credentialMgmtPreview"] = True
            pin_protocols = [1]

        return MetadataStatement(
            aaguid=aaguid,
            description=self.name,
            protocol_family="fido2",
            schema=3,
            key_protection=["software"],
            matcher_protection=["software"],
            attachment_hint=self.attachment_hint,
            tc_display=[],
            attestation_root_certificates=[attestation_root_certificate],
            upv=upv,
            authenticator_get_info={
                "versions": authenticator_versions,
                "extensions": ["credProtect", "hmac-secret"],
                "aaguid": aaguid.hex(),
                "options": options,
                "maxMsgSize": 3072,
                "pinUvAuthProtocols": pin_protocols,
                "maxCredentialCountInList": 10,
                "maxCredentialIdLength": 255,
                "transports": self.transports,
            },
            crypto_strength=0,
            # TODO: Do we want to set caDesc?
            user_verification_details=[
                [UVD_NONE],
                [UVD_PRESENCE_INTERNAL],
                [UVD_PASSCODE_EXTERNAL],
                [UVD_PRESENCE_INTERNAL, UVD_PASSCODE_EXTERNAL],
            ],
            # TODO: The spec says this should match the firmwareVersion
            # reported in authenticatorGetInfo, but that value does not exist
            authenticator_version=1,
            # TODO: Looks like this is only used for U2F.  Do we still need it?
            attestation_certificate_key_identifiers=None,
            # optional according to spec, but enforced by test suite
            # TODO: decide on icon, size and background
            icon=icon,
            # to be investigated
            authentication_algorithms=[
                "ed25519_eddsa_sha512_raw",
                "secp256r1_ecdsa_sha256_raw",
            ],
            public_key_alg_and_encodings=["cose", "ecc_x962_raw"],
            attestation_types=["basic_full"],
            is_key_restricted=True,
            is_fresh_user_verification_required=True,
        )


NK3AM = Authenticator(
    name="Nitrokey 3 AM",
    aaguid="2cd2f727-f6ca-44da-8f48-5c2e5da000a2",
    has_nfc=False,
    attestation_root_certificate="attestation/nk3am.der",
)

NK3XN = Authenticator(
    name="Nitrokey 3 xN",
    aaguid="ec99db19-cd1f-4c06-a2a9-940f17a6a30b",
    has_nfc=True,
    attestation_root_certificate="attestation/nk3xn.der",
)

NK3AM_TEST = Authenticator(
    name="Nitrokey 3 AM Test",
    aaguid="8bc54968-07b1-4d5f-b249-607f5d527da2",
    has_nfc=False,
    attestation_root_certificate="attestation/test.der",
    is_test=True,
)

NK3XN_TEST = Authenticator(
    name="Nitrokey 3 xN Test",
    aaguid="8bc54968-07b1-4d5f-b249-607f5d527da2",
    has_nfc=True,
    attestation_root_certificate="attestation/test.der",
    is_test=True,
)

if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("model", choices=["nk3am", "nk3xn"])
    parser.add_argument("--test", action="store_true")
    args = parser.parse_args()

    model = args.model
    if model == "nk3am":
        if args.test:
            authenticator = NK3AM_TEST
        else:
            authenticator = NK3AM
    elif model == "nk3xn":
        if args.test:
            authenticator = NK3XN_TEST
        else:
            authenticator = NK3XN
    else:
        raise ValueError(f"Unknown model {model}")

    ctap = CtapVersion.CTAP_2_1
    print(json.dumps(dict(authenticator.mds(ctap)), indent=4))
