import dataclasses
import datetime
import hashlib
from dataclasses import dataclass
from uuid import UUID
from pathlib import Path

from cryptography import x509

OID_AAGUID = x509.ObjectIdentifier("1.3.6.1.4.1.45724.1.1.4")


@dataclass(kw_only=True)
class Cache:
    serial_numbers: set[int] = dataclasses.field(default_factory=set)
    public_keys: set[bytes] = dataclasses.field(default_factory=set)


@dataclass(kw_only=True, frozen=True)
class Certificate:
    cert: x509.Certificate
    ext_aaguid: bytes | None

    def check_subject(self, *, cn_prefix: str, ou: str | None = None) -> None:
        cn = self.cert.subject.get_attributes_for_oid(x509.NameOID.COMMON_NAME)
        assert cn
        assert len(cn) == 1
        assert isinstance(cn[0].value, str)
        cn = cn[0].value
        assert cn.startswith(cn_prefix)

        attrs = [
            x509.NameAttribute(x509.NameOID.COUNTRY_NAME, "DE"),
            x509.NameAttribute(x509.NameOID.ORGANIZATION_NAME, "Nitrokey GmbH"),
        ]
        if ou is not None:
            attrs.append(x509.NameAttribute(x509.NameOID.ORGANIZATIONAL_UNIT_NAME, ou))
        attrs.append(x509.NameAttribute(x509.NameOID.COMMON_NAME, cn))

        assert self.cert.subject == x509.Name(attrs)

    def verify(self, issuer: "Certificate") -> None:
        self.cert.verify_directly_issued_by(issuer.cert)

    def hash(self) -> bytes:
        from cryptography.hazmat.primitives.serialization import Encoding

        return hashlib.sha256(self.cert.public_bytes(Encoding.DER)).digest()

    @staticmethod
    def from_pem_file(path: Path) -> "Certificate":
        cert = x509.load_pem_x509_certificate(path.read_bytes())

        now = datetime.datetime.now(tz=datetime.UTC)
        assert cert.not_valid_before_utc < now
        assert (cert.not_valid_after_utc - now) / datetime.timedelta(days=365) > 20

        ext_aaguid = None
        try:
            aaguid = cert.extensions.get_extension_for_oid(OID_AAGUID)
            assert not aaguid.critical
            assert isinstance(aaguid.value, x509.UnrecognizedExtension)
            ext_aaguid = aaguid.value.value[2:]
        except x509.ExtensionNotFound:
            pass

        return Certificate(cert=cert, ext_aaguid=ext_aaguid)


@dataclass(kw_only=True, frozen=True)
class Variant:
    fido_aaguid: str
    fido_batch_certs: list[str]

    def check(self, *, root_ca: Certificate, variant_dir: Path, cache: Cache) -> None:
        fido_ca = load_cert(variant_dir / "fido-ca.pem", cache)

        aaguid = UUID(self.fido_aaguid)
        check_fido_ca_cert(fido_ca=fido_ca, root_ca=root_ca)

        for i, hash in enumerate(self.fido_batch_certs):
            hash = bytes.fromhex(hash)
            fido_batch_cert = load_cert(variant_dir / f"fido-batch-{i + 1}.pem", cache)
            check_fido_batch_cert(
                batch_cert=fido_batch_cert, fido_ca=fido_ca, aaguid=aaguid, hash=hash
            )


@dataclass(kw_only=True, frozen=True)
class Model:
    variants: dict[str, Variant] | Variant

    def check(self, *, name: str, cache: Cache) -> None:
        model_dir = Path(name)

        root_ca = load_cert(model_dir / "root-ca.pem", cache)
        check_root_ca_cert(root_ca=root_ca)

        if isinstance(self.variants, Variant):
            self.variants.check(root_ca=root_ca, variant_dir=model_dir, cache=cache)
        else:
            for variant_name, variant in self.variants.items():
                variant_dir = model_dir / variant_name
                variant.check(root_ca=root_ca, variant_dir=variant_dir, cache=cache)


MODELS = {
    "nk3": Model(
        variants={
            "nk3am": Variant(
                fido_aaguid="2cd2f727-f6ca-44da-8f48-5c2e5da000a2",
                fido_batch_certs=[
                    "4c331d7af869fd1d8217198b917a33d1fa503e9778da7638504a64a438661ae0"
                ],
            ),
            "nk3xn": Variant(
                fido_aaguid="ec99db19-cd1f-4c06-a2a9-940f17a6a30b",
                fido_batch_certs=[
                    "aa1cb760c2879530e7d7fed3da75345d25774be9cfdbbcbd36fdee767025f34b"
                ],
            ),
        },
    ),
    "nkpk": Model(
        variants=Variant(
            fido_aaguid="9a03e537-4cbe-4a01-b2e2-242e0dd9a59b",
            fido_batch_certs=[
                "c7512dfcd15ffc5a7b4000e4898e5956ee858027794c5086cc137a02cd15d123"
            ],
        ),
    ),
}


def load_cert(path: Path, cache: Cache) -> Certificate:
    from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat

    cert = Certificate.from_pem_file(path)

    serial_number = cert.cert.serial_number
    assert serial_number not in cache.serial_numbers
    cache.serial_numbers.add(serial_number)

    public_key = cert.cert.public_key().public_bytes(
        Encoding.PEM, PublicFormat.SubjectPublicKeyInfo
    )
    assert public_key not in cache.public_keys
    cache.public_keys.add(public_key)

    return cert


def check_root_ca_cert(*, root_ca: Certificate) -> None:
    root_ca.check_subject(cn_prefix="Root")
    assert not root_ca.ext_aaguid


def check_fido_ca_cert(*, fido_ca: Certificate, root_ca: Certificate) -> None:
    fido_ca.verify(issuer=root_ca)
    fido_ca.check_subject(cn_prefix="FIDO CA")
    assert not fido_ca.ext_aaguid


def check_fido_batch_cert(
    *, batch_cert: Certificate, fido_ca: Certificate, aaguid: UUID, hash: bytes
) -> None:
    batch_cert.verify(issuer=fido_ca)
    batch_cert.check_subject(
        cn_prefix="Nitrokey FIDO Attestation", ou="Authenticator Attestation"
    )
    assert batch_cert.ext_aaguid == aaguid.bytes

    actual_hash = batch_cert.hash()
    assert actual_hash == hash


def run() -> None:
    cache = Cache()

    for name, model in MODELS.items():
        model.check(name=name, cache=cache)


if __name__ == "__main__":
    run()
