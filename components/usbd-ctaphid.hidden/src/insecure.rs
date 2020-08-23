//! WARNING: Using this needs a workaround due to
//! https://github.com/rust-lang/cargo/issues/5730
//!
//! The problem is that serde_cbor and bindgen's dependency rustc-hash
//! both use `byteorder`, but the latter activates the `std` feature,
//! breaking everything :/
//!
//! The workaround is to add the following in the application that actually uses this:
//!
//! ```ignore
//! [patch.crates-io]
//! rustc-hash = { git = "https://github.com/nickray/rustc-hash", branch = "nickray-remove-byteorder" }
//! ```
//!
//! # Goal:
//!
//! Here we implement a dumb FIDO2 device that just outputs
//! diagnostic messages using semihosting
//!
//! Maybe a better place is in a separate crate.
//!
//! Maybe also want to pull in dependencies like littlefs2, nisty, salty, ...
//!
//! Similar to littlefs2, the idea is to run test using this MVP implementation

use core::{
    convert::TryInto,
    ops::DerefMut,
};

use cortex_m_semihosting::hprintln;

#[cfg(feature = "logging")]
use funnel::info;
use cosey::PublicKey as CosePublicKey;

use heapless_bytes::Bytes;
use heapless::{
    Vec,
    String,
    consts,
};
use serde_indexed::{SerializeIndexed, DeserializeIndexed};

use crate::{
    authenticator::{
        self,
        Error,
        Result,
    },
    constants::{
        self,
        AUTHENTICATOR_DATA_LENGTH_BYTES,
    },
    types::{
        AssertionResponse,
        AssertionResponses,
        AttestationObject,
        AttestationStatement,
        AttestedCredentialData,
        AuthenticatorData,
        AuthenticatorInfo,
        GetAssertionParameters,
        MakeCredentialParameters,
        // NoneAttestationStatement,
        PackedAttestationStatement,
        // PublicKeyCredentialUserEntity,
    },
};

// TODO: generate this in a clean way. e.g. python cryptography SUX
pub const SOLO_HACKER_ATTN_CERT: [u8; 511] = *include_bytes!("solo-hacker-attn-cert.der");
pub const SOLO_HACKER_ATTN_KEY: [u8; 32] = *include_bytes!("solo-hacker-attn-key.le.raw");

pub enum Keypair {
    Ed25519(salty::Keypair),
    P256(nisty::Keypair),
}

impl Keypair {
    pub fn as_cose_public_key(&self) -> cosey::PublicKey {
        match self {
            Self::P256(keypair) => {
                let cose_variant: nisty::CosePublicKey = keypair.public.clone().into();
                cose_variant.into()
            },
            Self::Ed25519(keypair) => {
                let cose_variant: salty::CosePublicKey = keypair.public.clone().into();
                cose_variant.into()
            }
        }
    }

    pub fn asn1_sign_prehashed(&self, digest: &[u8; 32]) -> Bytes<consts::U72> {
        match self {
            Self::Ed25519(keypair) => {
                let sig_fixed = keypair.sign(digest).to_bytes();
                Bytes::<consts::U72>::try_from_slice(&sig_fixed).unwrap()
            },

            Self::P256(keypair) => {
                keypair.sign_prehashed(digest).to_asn1_der()
            },
        }
    }
}

pub struct InsecureRamAuthenticator {
    aaguid: Bytes<consts::U16>,
    master_secret: [u8; 32],
    signature_count: u32,
}

impl InsecureRamAuthenticator {
}

impl Default for InsecureRamAuthenticator {
    fn default() -> Self {
        InsecureRamAuthenticator {
            aaguid: Bytes::try_from_slice(b"AAGUID0123456789").unwrap(),
            // Haaha. See why this is called an "insecure" authenticator? :D
            master_secret: [37u8; 32],
            signature_count: 123,
        }
    }
}

// solo-c uses CredentialId:
// * rp_id_hash:
// * (signature_)counter: to be able to sort by recency descending
// * nonce
// * authentication tag
//
// For resident keys, it uses (CredentialId, UserEntity)
#[derive(Clone,Debug,Eq,PartialEq,SerializeIndexed,DeserializeIndexed)]
pub struct CredentialInner {
    pub user_id: Bytes<consts::U64>,
    pub alg: i8,
    pub seed: Bytes<consts::U32>,
}

impl authenticator::Api for InsecureRamAuthenticator {
    fn get_assertions(&mut self, params: &GetAssertionParameters) -> Result<AssertionResponses>
    {
        // 1. locate all eligible credentials
        // if params.allow_list.len() != 1 {
        //     return Err(Error::
        // let number_of_credentials: u32 = ...

        // 2-4. PIN stuff

        // 5. process options

        // 6. process extensions

        // 7. collect user consent

        // 8. if no credentials were located in step 1
        // muy importante: not before step 7!
        // if number_of_credentials == 0 {
        //     return Err(Error::NoCredentials);
        // }

        // 9. if more than one credential found,
        // order by creation timestampe descending

        // 10. no display:

        // 11. has display:

        // 12. sign client data hash and auth data with selected credential

        // AND NOW SHORTCUT
        if params.allow_list.len() == 0 {
            return Err(Error::NoCredentials);
        }

        assert!(params.allow_list.len() == 1);
        // let number_of_credentials: u32 = 1;

        let mut cloned_credential_id = params.allow_list[0].id.clone();
        // hprintln!("attempting deserialize: {:?}", &cloned_credential_id).ok();
        let credential_inner: CredentialInner =
            ctapcbor::de::from_bytes(cloned_credential_id.deref_mut()).unwrap();
        // hprintln!("credential inner: {:?}", &credential_inner).ok();

        //// generate authenticator data
        //let attested_credential_data = AttestedCredentialData {
        //    aaguid: self.aaguid.clone(),
        //    credential_id,
        //    credential_public_key,
        //};
        //// hprintln!("attested credential data = {:?}", attested_credential_data).ok();

        //// flags:
        ////
        //// USER_PRESENT = 0x01
        //// USER_VERIFIED = 0x04
        //// ATTESTED = 0x40
        //// EXTENSION_DATA = 0x80
        //let auth_data = AuthenticatorData {
        //    rp_id_hash: Bytes::<consts::U32>::from({
        //        let mut bytes = Vec::<u8, consts::U32>::new();
        //        bytes.extend_from_slice(&nisty::prehash(&params.rp.id.as_str().as_bytes())).unwrap();
        //        bytes
        //    }),
        //    flags: 0x40,
        //    // flags: 0x0,
        //    sign_count: 123,
        //    attested_credential_data: Some(attested_credential_data.serialize()),
        //    // attested_credential_data: None,
        //};

        // now sign it. what to do?
        // 1. sha-256-digest(&authenticator_data || client_data_hash) -> digest
        // 2. sign(digest) -> signature-bytes
        // 3. der-encode(signature-bytes) -> signature-der (for this, cf. ctap_encode_der_sig)

        // let credential_public_key = if credential_inner.alg == -8 {
        let keypair = if credential_inner.alg == -8 {
            // Ed25519
            // hprintln!("logging in with 25519").ok();
            Keypair::Ed25519(salty::Keypair::from(&credential_inner.seed.as_ref().try_into().unwrap()))
        } else {
            // NIST P-256
            // hprintln!("logging in with NIST").ok();
            let seed_array: [u8; 32] = credential_inner.seed.as_ref().try_into().unwrap();
            Keypair::P256(nisty::Keypair::generate_patiently(&seed_array))
        };

        // let attested_credential_data = AttestedCredentialData {
        //     aaguid: self.aaguid.clone(),
        //     credential_id: cloned_credential_id,
        //     credential_public_key: keypair.serialize_public_key(),
        // };
        let auth_data = AuthenticatorData {
            rp_id_hash: Bytes::<consts::U32>::from({
                let mut bytes = Vec::<u8, consts::U32>::new();
                bytes.extend_from_slice(&nisty::prehash(&params.rp_id.as_str().as_bytes())).unwrap();
                bytes
            }),
            // TODO: what goes here?
            flags: 0x01, // | 0x40,
            // flags: 0x0,
            sign_count: self.signature_count,
            attested_credential_data: None, //Some(attested_credential_data.serialize()),
            // attested_credential_data: None,
        };
        self.signature_count += 1;
        // hprintln!("auth_data = {:?}", &auth_data).ok();
        let serialized_auth_data = auth_data.serialize();

        use sha2::digest::Digest;
        let mut hash = sha2::Sha256::new();
        hash.input(&serialized_auth_data);
        hash.input(&params.client_data_hash);
        let digest: [u8; 32] = hash.result().try_into().unwrap();
        // data.into()
        let sig = if credential_inner.alg == -8 {
            let mut buf = [0u8; AUTHENTICATOR_DATA_LENGTH_BYTES + 32];
            let auth_data_size = serialized_auth_data.len();
            buf[..auth_data_size].copy_from_slice(&serialized_auth_data);

            // hprintln!("auth_data_size = {}", auth_data_size).ok();
            // hprintln!("self.auth_data = {:?}", &serialized_auth_data).ok();
            // buf[auth_data_size..][..32].copy_from_slice(&params.client_data_hash);
            // hprintln!("client_param = {:?}", &params.client_data_hash).ok();
            buf[auth_data_size..][..params.client_data_hash.len()].copy_from_slice(&params.client_data_hash);

            let sig_fixed = match keypair {
                Keypair::Ed25519(keypair) => {
                    keypair.sign(&buf[..auth_data_size + params.client_data_hash.len()]).to_bytes()
                },
                _ => { unreachable!(); },
            };
            Bytes::<consts::U72>::try_from_slice(&sig_fixed).unwrap()
        } else {
            // let sig = keypair.asn1_sign_prehashed(&digest);
            keypair.asn1_sign_prehashed(&digest)
        };

        // pub user: Option<PublicKeyCredentialUserEntity>,
        // pub auth_data: Bytes<AUTHENTICATOR_DATA_LENGTH>,
        // pub signature: Bytes<SIGNATURE_LENGTH>,
        // pub credential: Option<PublicKeyCredentialDescriptor>,
        // pub number_of_credentials: Option<u32>,
        let response = AssertionResponse {
            user: None, //Some(PublicKeyCredentialUserEntity::from(credential_inner.user_id.clone())),
            // TODO!
            auth_data: serialized_auth_data,
            // TODO!
            signature: sig,
            credential: None, //Some(params.allow_list[0].clone()),
            number_of_credentials: None, // Some(1),
            // number_of_credentials: Some(1),
        };

        let mut responses = AssertionResponses::new();
        responses.push(response).unwrap();
        hprintln!("returning responses").ok();

        Ok(responses)


    }

    fn make_credential(&mut self, params: &MakeCredentialParameters) -> Result<AttestationObject> {
        // 0. Some general checks?

        // current solo does this
        if params.client_data_hash.len() != 32 {
            return Err(Error::InvalidLength);
        }

        // 1. Check excludeList
        // TODO

        // 2. check pubKeyCredParams algorithm is valid COSE identifier and supported
        let mut supported_algorithm = false;
        let mut eddsa = false;
        // let mut es256 = false;
        for param in params.pub_key_cred_params.iter() {
            match param.alg {
                -7 => { /*es256 = true;*/ supported_algorithm = true; },
                -8 => { eddsa = true; supported_algorithm = true; },
                _ => {},
            }
        }
        // TODO: temporary, remove!!
        // eddsa = false;
        if !supported_algorithm {
            return Err(Error::UnsupportedAlgorithm);
        }

        // 3. check for known but unsupported options
        match &params.options {
            Some(ref options) => {
                if Some(true) == options.rk {
                    return Err(Error::UnsupportedOption);
                }
                if Some(true) == options.uv {
                    return Err(Error::UnsupportedOption);
                }
            },
            _ => {},
        }

        // 4. optionally, process extensions

        // 5-7. pinAuth handling
        // TODO

        // 8. request user presence (blink LED, or show user + rp on display if present)

        // 9. generate new key pair \o/
        //
        // We do it quick n' dirty here because YOLO
        let mut hash = salty::Sha512::new();
        hash.update(&self.master_secret);
        hash.update(&params.rp.id.as_str().as_bytes());
        hash.update(&params.user.id);
        let digest: [u8; 64] = hash.finalize();
        let seed = nisty::prehash(&digest);

        // let keypair = if eddsa {
        let keypair = if eddsa {
            // prefer Ed25519
            #[cfg(feature = "logging")]
            info!("making Ed25519 credential, woo!").ok();
            Keypair::Ed25519(salty::Keypair::from(&seed))
        } else {
            #[cfg(feature = "logging")]
            info!("making P256 credential, eww!").ok();
            Keypair::P256(nisty::Keypair::generate_patiently(&seed))
        };

        let credential_public_key: CosePublicKey = keypair.as_cose_public_key();

        // hprintln!("serialized public_key: {:?}", &credential_public_key).ok();

        // 10. if `rk` option is set, attempt to store it
        // -> ruled out by above

        // 11. generate attestation statement.
        // For now, only "none" format, which has serialized "empty map" (0xa0) as its statement

        // return the attestation object
        // WARNING: another reason this is highly insecure, we return the seed
        // as credential ID ^^
        // TODO: do some AEAD based on xchacha20, later reject tampered/invalid credential IDs
        let credential_inner = CredentialInner {
            user_id: params.user.id.clone(),
            alg: if eddsa { -8 } else { -7 },
            seed: Bytes::try_from_slice(&seed).unwrap(),
        };
        // hprintln!("credential inner: {:?}", &credential_inner);
                        // let writer = serde_cbor::ser::SliceWrite::new(&mut self.buffer[1..]);
                        // let mut ser = serde_cbor::Serializer::new(writer)
                        //     .packed_format()
                        //     .pack_starting_with(1)
                        //     .pack_to_depth(2)
                        // ;

                        // attestation_object.serialize(&mut ser).unwrap();

                        // let writer = ser.into_inner();
                        // let size = 1 + writer.bytes_written();

        let credential_id = Bytes::<consts::U128>::from_serialized(&credential_inner);
        // hprintln!("credential_id: {:?}", &credential_id).ok();
        // let mut credential_id = Bytes::<consts::U128>::new();
        // credential_id.extend_from_slice(&seed).unwrap();

        let attested_credential_data = AttestedCredentialData {
            aaguid: self.aaguid.clone(),
            credential_id,
            credential_public_key,
        };
        // hprintln!("attested credential data = {:?}", attested_credential_data).ok();

        // flags:
        //
        // USER_PRESENT = 0x01
        // USER_VERIFIED = 0x04
        // ATTESTED = 0x40
        // EXTENSION_DATA = 0x80
        let auth_data = AuthenticatorData {
            rp_id_hash: Bytes::<consts::U32>::from({
                let mut bytes = Vec::<u8, consts::U32>::new();
                bytes.extend_from_slice(&nisty::prehash(&params.rp.id.as_str().as_bytes())).unwrap();
                bytes
            }),
            flags: 0x01 | 0x40,
            // flags: 0x0,
            sign_count: self.signature_count,
            attested_credential_data: Some(attested_credential_data.serialize()),
            // attested_credential_data: None,
        };
        self.signature_count += 1;
        // hprintln!("auth data = {:?}", &auth_data).ok();

        let serialized_auth_data = auth_data.serialize();

        // // NONE
        // let fmt = String::<consts::U32>::from("none");
        // let att_stmt = AttestationStatement::None(NoneAttestationStatement {}); // "none" attestion requires empty statement

        // PACKED
        use sha2::digest::Digest;
        let mut hash = sha2::Sha256::new();
        hash.input(&serialized_auth_data);
        hash.input(&params.client_data_hash);
        let digest: [u8; 32] = hash.result().try_into().unwrap();
        // data.into()
        let attn_keypair = Keypair::P256(nisty::Keypair::try_from_bytes(&SOLO_HACKER_ATTN_KEY).unwrap());
        let sig = attn_keypair.asn1_sign_prehashed(&digest);

        let mut packed_attn_stmt = PackedAttestationStatement {
            alg: -7,
            sig,
            x5c: Vec::new(),
        };
        packed_attn_stmt.x5c.push(Bytes::try_from_slice(&SOLO_HACKER_ATTN_CERT).unwrap()).unwrap();

        let fmt = String::<consts::U32>::from("packed");
        let att_stmt = AttestationStatement::Packed(packed_attn_stmt);


        let attestation_object = AttestationObject {
            fmt,
            auth_data: serialized_auth_data,
            att_stmt,
        };

        Ok(attestation_object)
    }

    fn get_info(&self) -> AuthenticatorInfo {

        use core::str::FromStr;
        let mut versions = Vec::<String<consts::U8>, consts::U2>::new();
        // versions.push(String::from_str("U2F_V2").unwrap()).unwrap();
        versions.push(String::from_str("FIDO_2_0").unwrap()).unwrap();

        AuthenticatorInfo {
            versions,
            aaguid: self.aaguid.clone(),
            max_msg_size: Some(constants::MESSAGE_SIZE),
            ..AuthenticatorInfo::default()
        }
    }

    fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}

#[macro_export]
macro_rules! insecure_ram_authenticator {
    (api=$AuthenticatorApi:path, ctap_options=$CtapOptions:ty) => {
        struct InsecureRamAuthenticator {
        }

        impl InsecureRamAuthenticator {
        }

        impl $AuthenticatorApi for InsecureRamAuthenticator {
            fn get_info(&self) -> AuthenticatorInfo {

                AuthenticatorInfo {
                    versions: &["FIDO_2_0"], // &["U2F_V2", "FIDO_2_0"],
                    extensions: None, // Some(&["hmac-secret"]),
                    aaguid: b"AAGUID0123456789",
                    // options: None, // Some(CtapOptions::default()),
                    options: Some(<$CtapOptions>::default()),
                    // max_msg_size: Some(MESSAGE_SIZE),
                    max_msg_size: Some(7609),
                    pin_protocols: None, // Some(&[1]),
                };

            }
        }

    }
}
