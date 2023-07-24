#![no_std]

use embedded_hal::blocking::delay::DelayUs;
use hex_literal::hex;
use iso7816::{Instruction, Status};
use se050::{
    se050::{
        commands::{
            CipherDecryptInit, CipherEncryptInit, CipherFinal, CipherOneShotDecrypt,
            CipherOneShotEncrypt, CipherUpdate, CreateCipherObject, CreateDigestObject,
            CreateSession, CreateSignatureObject, DeleteAll, DeleteCryptoObj, DeleteSecureObject,
            DigestFinal, DigestInit, DigestOneShot, DigestUpdate, EcdsaSign, EcdsaVerify,
            EddsaSign, EddsaVerify, GenRsaKey, GetRandom, MacGenerateFinal, MacGenerateInit,
            MacOneShotGenerate, MacOneShotValidate, MacUpdate, MacValidateFinal, MacValidateInit,
            ReadEcCurveList, ReadIdList, ReadObject, RsaSign, RsaVerify, VerifySessionUserId,
            WriteBinary, WriteEcKey, WriteSymmKey, WriteUserId,
        },
        policies::{ObjectAccessRule, ObjectPolicyFlags, Policy, PolicySet},
        CipherMode, CryptoObjectId, Digest, EcCurve, EcDsaSignatureAlgo, MacAlgo, ObjectId,
        P1KeyType, ProcessSessionCmd, RsaSignatureAlgo, Se050, Se050Result, SymmKeyType,
    },
    t1::I2CForT1,
};

delog::generate_macros!();

const BUFFER_LEN: usize = 300;

pub struct Card<Twi, D> {
    se: Se050<Twi, D>,
    enabled: bool,
    failed_enable: Option<Status>,
    buf: [u8; BUFFER_LEN],
}

macro_rules! command {
    ($e:expr; $msg:literal) => {{
        debug_now!($msg);
        let __res = $e;
        debug_now!("Got res: {:?}", __res);
        __res?
    }};
}

impl<Twi: I2CForT1, D: DelayUs<u32>> Card<Twi, D> {
    pub fn new(twi: Twi, se_address: u8, delay: D) -> Self {
        Card {
            se: Se050::new(twi, se_address, delay),
            enabled: false,
            failed_enable: None,
            buf: [b'a'; BUFFER_LEN],
        }
    }

    fn handle<const C: usize, const R: usize>(
        &mut self,
        command: &iso7816::Command<C>,
        reply: &mut heapless::Vec<u8, R>,
    ) -> Result<(), Status> {
        if let Some(status) = self.failed_enable {
            error!("Failed enable");
            return Err(status);
        }
        debug_now!("SE050 test command: {:?}", command);
        if command.instruction() == Instruction::Select {
            if !self.enabled {
                debug_now!("Running Enable");
                let res = self.se.enable();
                error!("Got res: {:?}", res);
                if let Err(err) = res {
                    self.failed_enable = Some(err.into());
                    return Err(err.into());
                }
                self.enabled = true;
            } else {
                debug_now!("Already enabled, ditching");
            }
            return Ok(());
        }

        match u8::from(command.instruction()) {
            0x84 => {
                let mut buf = [b'a'; BUFFER_LEN];
                let len = command.expected();
                let res = command!(self.se.run_command(&GetRandom {length: (len as u16).into()}, &mut buf); "Running get random");
                reply.extend_from_slice(res.data).unwrap();
                if res.data == &[b'a'; BUFFER_LEN][..len] {
                    debug_now!("Failed to get random");
                    return Err(Status::UnspecifiedNonpersistentExecutionError);
                }
                self.buf[..len].copy_from_slice(res.data)
            }
            0xD1 => {
                let data = &hex!("31323334");
                let mut buf = [0; 100];
                command!(self.se.run_command(&WriteUserId {
                    policy: None,
                    max_attempts: None,
                    object_id: ObjectId::FACTORY_RESET,
                    data ,
                },&mut buf); "creating user id");
                let session = command!(self.se.run_command(&CreateSession {
                    object_id: ObjectId::FACTORY_RESET,
                }, &mut buf); "Creating session");
                command!(self.se.run_command(&ProcessSessionCmd {
                    session_id: session.session_id,
                    apdu: VerifySessionUserId { user_id: data},
                },&mut buf); "Verifying user id");
                command!(self.se.run_command(&ProcessSessionCmd {
                    session_id: session.session_id,
                    apdu: DeleteAll {},
                },&mut buf); "Performing factory reset");
            }
            0xD2 => {
                let data = &hex!("31323334");
                let mut buf = [0; 100];
                let session = command!(self.se.run_command(&CreateSession {
                    object_id: ObjectId::FACTORY_RESET,
                }, &mut buf); "Creating session");
                command!(self.se.run_command(&ProcessSessionCmd {
                    session_id: session.session_id,
                    apdu: VerifySessionUserId { user_id: data},
                },&mut buf); "Verifying user id");
                command!(self.se.run_command(&ProcessSessionCmd {
                    session_id: session.session_id,
                    apdu: DeleteAll {},
                },&mut buf); "Performing factory reset");
            }
            0xD3 => {
                let mut buf = [0; 200];
                let object_ids = command!(self.se.run_command(&ReadIdList{offset:0.into(),filter: se050::se050::SecureObjectFilter::All}, &mut buf); "Getting object list");
                reply.extend_from_slice(object_ids.ids).ok();
            }
            0xD4 => {
                let mut buf = [b'a'; 400];
                let len = command.expected();
                let object_id = ObjectId(hex!("01020304"));
                let policy = &[Policy {
                    object_id: ObjectId::INVALID,
                    access_rule: ObjectAccessRule::from_flags(
                        ObjectPolicyFlags::ALLOW_DELETE | ObjectPolicyFlags::ALLOW_READ,
                    ),
                }];
                command!(self.se.run_command(
                    &WriteBinary {
                        transient: false,
                        policy: Some(PolicySet(policy)),
                        object_id,
                        offset: None,
                        file_length: Some((len as u16).into()),
                        data: Some(&self.buf[..len]),
                    }, &mut buf);
                    "Running write_binary"
                );
                reply.extend_from_slice(&self.buf[..len]).ok();
            }
            0xD5 => {
                let mut buf = [0; 400];
                let len = command.expected();
                let object_id = ObjectId(hex!("01020304"));
                let res = command!(self.se.run_command(
                    &ReadObject {
                        object_id,
                        offset: Some(0.into()),
                        length: Some((len as u16).into()),
                        rsa_key_component: None,
                    }, &mut buf);
                    "Running read_binary"
                );
                assert_eq!(res.data, &self.buf[..len]);
                reply.extend_from_slice(res.data).ok();
            }
            0xD6 => {
                let mut buf = [0; 200];
                let object_id = ObjectId(hex!("01020304"));
                command!(self.se.run_command(
                    &DeleteSecureObject {
                        object_id,
                    }, &mut buf);
                    "Running delete_binary"
                );
            }
            0xD7 => {
                command!(self.se.create_and_set_curve(EcCurve::NistP256); "Creating curve NistP256");
            }
            0xD8 => {
                let mut buf = [0; 200];
                let mut buf2 = [0; 200];
                let object_id = ObjectId(hex!("01223344"));
                command!(self.se.run_command(
                    &WriteEcKey {
                        transient: false,
                        is_auth: false,
                        key_type: Some(P1KeyType::KeyPair),
                        policy: None,
                        max_attempts: None,
                        object_id,
                        curve: Some(EcCurve::NistP256),
                        private_key: None,
                        public_key: None,
                    },
                    &mut buf,
                ); "Creating ec key");
                let res = command!(self.se.run_command(
                    &EcdsaSign {
                        key_id: object_id,
                        data: &[52; 32],
                        algo: EcDsaSignatureAlgo::Sha256,
                    },
                    &mut buf
                ); "Runing signature");
                let res = command!(self.se.run_command(
                    &EcdsaVerify {
                        key_id: object_id,
                        data: &[52; 32],
                        algo: EcDsaSignatureAlgo::Sha256,
                        signature: res.signature
                    },
                    &mut buf2
                ); "Runing verifcation");
                if res.result == Se050Result::Success {
                    reply.push(0x01).unwrap();
                } else {
                    reply.push(0x02).unwrap();
                }
            }
            0xD9 => {
                let mut buf = [0; 200];
                let res = command!(self.se.run_command(&ReadEcCurveList {}, &mut buf); "Reading EC curve list");
                reply.extend_from_slice(res.ids).ok();
            }
            0xDA => {
                let mut buf = [0; 200];
                let mut buf2 = [0; 200];
                let object_id = ObjectId(hex!("01223344"));
                command!(self.se.run_command(
                    &WriteEcKey {
                        transient: false,
                        is_auth: false,
                        key_type: Some(P1KeyType::KeyPair),
                        policy: None,
                        max_attempts: None,
                        object_id,
                        curve: Some(EcCurve::IdEccEd25519),
                        private_key: None,
                        public_key: None,
                    },
                    &mut buf,
                ); "Creating ec key");
                let res = command!(self.se.run_command(
                    &EddsaSign {
                        key_id: object_id,
                        data: &[52; 32],
                    },
                    &mut buf
                ); "Runing signature");
                let res = command!(self.se.run_command(
                    &EddsaVerify     {
                        key_id: object_id,
                        data: &[52; 32],
                        signature: res.signature
                    },
                    &mut buf2
                ); "Runing verifcation");
                if res.result == Se050Result::Success {
                    reply.push(0x01).unwrap();
                } else {
                    reply.push(0x02).unwrap();
                }
            }
            0xDB => {
                let mut buf = [0; 1000];
                let mut buf2 = [0; 1000];
                let object_id = ObjectId(hex!("01223344"));
                command!(self.se.run_command(
                    &GenRsaKey {
                        transient: false,
                        is_auth: false,
                        policy: None,
                        max_attempts: None,
                        object_id,
                        key_size: Some(2048.into()),
                    },
                    &mut buf,
                ); "Creating RSA key");
                let res = command!(self.se.run_command(
                    &RsaSign {
                        key_id: object_id,
                        data: &[52; 32],
                        algo: RsaSignatureAlgo::RsaSha256Pkcs1,
                    },
                    &mut buf
                ); "Runing signature");
                let res = command!(self.se.run_command(
                    &RsaVerify     {
                        key_id: object_id,
                        data: &[52; 32],
                        algo: RsaSignatureAlgo::RsaSha256Pkcs1,
                        signature: res.signature
                    },
                    &mut buf2
                ); "Runing verifcation");
                if res.result == Se050Result::Success {
                    reply.push(0x01).unwrap();
                } else {
                    reply.push(0x02).unwrap();
                }
            }
            0xDD => {
                let mut buf = [0; 1000];
                let mut buf2 = [0; 1000];
                let object_id = ObjectId(hex!("02334455"));
                command!(self.se.run_command(
                    &GenRsaKey {
                        transient: false,
                        is_auth: false,
                        policy: None,
                        max_attempts: None,
                        object_id,
                        key_size: Some(4096.into()),
                    },
                    &mut buf,
                ); "Creating RSA key");
                let res = command!(self.se.run_command(
                    &RsaSign {
                        key_id: object_id,
                        data: &[52; 32],
                        algo: RsaSignatureAlgo::RsaSha512Pkcs1,
                    },
                    &mut buf
                ); "Runing signature");
                let res = command!(self.se.run_command(
                    &RsaVerify     {
                        key_id: object_id,
                        data: &[52; 32],
                        algo: RsaSignatureAlgo::RsaSha512Pkcs1,
                        signature: res.signature
                    },
                    &mut buf2
                ); "Runing verifcation");
                if res.result == Se050Result::Success {
                    reply.push(0x01).unwrap();
                } else {
                    reply.push(0x02).unwrap();
                }
            }
            0xDC => {
                let Ok(count) = (&**command.data()).try_into() else {
                    error!("Bad count data");
                    return Err(Status::IncorrectDataParameter);
                };
                let retries = u32::from_be_bytes(count);
                self.se.set_t1_retry_count(retries);
            }

            0xDE => {
                let mut buf = [0; 1000];
                let mut buf2 = [0; 1000];
                let plaintext_data = [2; 32 * 15];
                let key_id = ObjectId(hex!("03445566"));
                let cipher_id = CryptoObjectId(hex!("0123"));
                let key = [0x42; 32];
                let iv = [0xFF; 16];
                command!(self.se.run_command(&WriteSymmKey {
                    transient: true,
                    is_auth: false,
                    key_type: SymmKeyType::Aes,
                    policy: None,
                    max_attempts: None,
                    object_id: key_id,
                    kek_id: None,
                    value: &key,
                }, &mut buf); "writing key");
                let ciphertext1 = command!(self.se.run_command(& CipherOneShotEncrypt {
                    key_id,
                    mode: CipherMode::AesCtr,
                    plaintext: &plaintext_data,
                    initialization_vector: Some(&iv),
                }, &mut buf); "one shot encrypt");
                let plaintext1 = command!(self.se.run_command(& CipherOneShotDecrypt {
                    key_id,
                    mode: CipherMode::AesCtr,
                    ciphertext: &ciphertext1.ciphertext,
                    initialization_vector: Some(&iv),
                }, &mut buf2); "one shot decrypt");
                assert_eq!(plaintext1.plaintext, plaintext_data);
                command!(self.se.run_command(&CreateCipherObject {
                    id: cipher_id,
                    subtype: CipherMode::AesCtr,
                }, &mut buf2); "Creating cipher object");
                command!(self.se.run_command(& CipherEncryptInit {
                    key_id,
                    initialization_vector: Some(&iv),
                    cipher_id,
                }, &mut buf2); "init encrypt");
                let ciphertext2 = command!(self.se.run_command(& CipherUpdate {
                    cipher_id,
                    data: &plaintext_data[0..32*10],
                }, &mut buf2); "init encrypt");
                reply.extend_from_slice(&ciphertext2.data).ok();
                let ciphertext3 = command!(self.se.run_command(& CipherUpdate {
                    cipher_id,
                    data: &plaintext_data[32*10..][..32*5],
                }, &mut buf2); "init encrypt");
                reply.extend_from_slice(&ciphertext3.data).ok();
                let ciphertext4 = command!(self.se.run_command(& CipherFinal {
                    cipher_id,
                    data: &plaintext_data[32*15..],
                }, &mut buf2); "init encrypt");
                reply.extend_from_slice(&ciphertext4.data).ok();
                command!(self.se.run_command(&DeleteCryptoObj {
                    id: cipher_id,
                }, &mut buf2); "Deleting cipher object");
                reply.extend_from_slice(&[0x42; 16]).ok();
                command!(self.se.run_command(&CreateCipherObject {
                    id: cipher_id,
                    subtype: CipherMode::AesCtr,
                }, &mut buf2); "Creating cipher object");
                command!(self.se.run_command(& CipherDecryptInit {
                    key_id,
                    initialization_vector: Some(&iv),
                    cipher_id,
                }, &mut buf2); "init encrypt");
                let ciphertext2 = command!(self.se.run_command(& CipherUpdate {
                    cipher_id,
                    data: &ciphertext1.ciphertext[0..32*10],
                }, &mut buf2); "encrypt update");
                reply.extend_from_slice(&ciphertext2.data).ok();
                let ciphertext3 = command!(self.se.run_command(& CipherUpdate {
                    cipher_id,
                    data: &ciphertext1.ciphertext[32*10..][..32*5],
                }, &mut buf2); "encrypt update");
                reply.extend_from_slice(&ciphertext3.data).ok();
                let ciphertext4 = command!(self.se.run_command(& CipherFinal {
                    cipher_id,
                    data: &ciphertext1.ciphertext[32*15..],
                }, &mut buf2); "encrypt final");
                reply.extend_from_slice(&ciphertext4.data).ok();
                command!(self.se.run_command(&DeleteCryptoObj {
                    id: cipher_id,
                }, &mut buf2); "Deleting cipher object");
                command!(self.se.run_command(&DeleteSecureObject { object_id: key_id }, &mut buf2); "deleting");
            }
            0xDF => {
                let mut buf = [0; 1000];
                let mut buf2 = [0; 1000];
                let plaintext_data = [2; 32 * 15];
                let key_id = ObjectId(hex!("03445566"));
                let mac_id = CryptoObjectId(hex!("0123"));
                let key = [0x42; 32];
                command!(self.se.run_command(&WriteSymmKey {
                    transient: false,
                    is_auth: false,
                    key_type: SymmKeyType::Hmac,
                    policy: None,
                    max_attempts: None,
                    object_id: key_id,
                    kek_id: None,
                    value: &key,
                }, &mut buf); "writing key");
                let tag1 = command!(self.se.run_command(& MacOneShotGenerate {
                    key_id,
                    data: &plaintext_data,
                    algo: MacAlgo::HmacSha256
                }, &mut buf); "one shotd generate");
                reply.extend_from_slice(tag1.tag).ok();
                let res = command!(self.se.run_command(& MacOneShotValidate {
                    key_id,
                    algo: MacAlgo::HmacSha256,
                    data: &plaintext_data,
                    tag: tag1.tag,
                }, &mut buf2); "one shot decrypt");
                if res.result == Se050Result::Success {
                    reply.extend_from_slice(&[0x01; 16]).unwrap();
                } else {
                    reply.extend_from_slice(&[0x02; 16]).unwrap();
                }
                command!(self.se.run_command(&CreateSignatureObject {
                    id: mac_id,
                    subtype: MacAlgo::HmacSha256,
                }, &mut buf2); "Creating mac object");
                command!(self.se.run_command(& MacGenerateInit {
                    key_id,
                    mac_id,
                }, &mut buf2); "init generate");
                command!(self.se.run_command(& MacUpdate {
                    mac_id,
                    data: &plaintext_data[0..32*10],
                }, &mut buf2); "update");
                command!(self.se.run_command(& MacUpdate {
                    mac_id,
                    data: &plaintext_data[32*10..][..32*5],
                }, &mut buf2); "update");
                let tag2 = command!(self.se.run_command(&MacGenerateFinal {
                    mac_id,
                    data: &plaintext_data[32*15..],
                }, &mut buf2); "generate final");
                assert_eq!(tag2.tag, tag1.tag);
                command!(self.se.run_command(&DeleteCryptoObj {
                    id: mac_id,
                }, &mut buf); "deleting mac object");

                command!(self.se.run_command(&CreateSignatureObject {
                    id: mac_id,
                    subtype: MacAlgo::HmacSha256,
                }, &mut buf); "Creating mac object");
                command!(self.se.run_command(& MacValidateInit {
                    key_id,
                    mac_id,
                }, &mut buf); "init validate");
                command!(self.se.run_command(& MacUpdate {
                    mac_id,
                    data: &plaintext_data[0..32*10],
                }, &mut buf); "update");
                command!(self.se.run_command(& MacUpdate {
                    mac_id,
                    data: &plaintext_data[32*10..][..32*5],
                }, &mut buf); "update");
                let res2 = command!(self.se.run_command(&MacValidateFinal {
                    mac_id,
                    data: &plaintext_data[32*15..],
                    tag: tag2.tag,
                }, &mut buf); "validate final");
                if res2.result == Se050Result::Success {
                    reply.extend_from_slice(&[0x01; 16]).unwrap();
                } else {
                    reply.extend_from_slice(&[0x02; 16]).unwrap();
                }
                command!(self.se.run_command(&DeleteCryptoObj {
                    id: mac_id,
                }, &mut buf2); "Deleting mac object");
                command!(self.se.run_command(&DeleteSecureObject { object_id: key_id }, &mut buf2); "deleting");
            }
            0xE1 => {
                let mut buf = [0; 1000];
                let mut buf2 = [0; 1000];
                let plaintext_data = [2; 32 * 15];
                let digest_id = CryptoObjectId(hex!("0123"));
                let digest1 = command!(self.se.run_command(& DigestOneShot {
                    algo: Digest::Sha256,
                    data: &plaintext_data,
                }, &mut buf); "one shot digest");
                reply.extend_from_slice(digest1.digest).ok();
                reply.extend_from_slice(&[0x42; 16]).ok();
                command!(self.se.run_command(&CreateDigestObject {
                    id: digest_id,
                    subtype: Digest::Sha256,
                }, &mut buf2); "Creating digest object");
                command!(self.se.run_command(& DigestInit {
                   digest_id,
                }, &mut buf2); "init digest");
                command!(self.se.run_command(& DigestUpdate {
                    digest_id,
                    data: &plaintext_data[0..32*10],
                }, &mut buf2); "update");
                command!(self.se.run_command(& DigestUpdate {
                    digest_id,
                    data: &plaintext_data[32*10..][..32*5],
                }, &mut buf2); "update");
                let digest2 = command!(self.se.run_command(&DigestFinal {
                    digest_id,
                    data: &plaintext_data[32*15..],
                }, &mut buf2); "generate final");
                reply.extend_from_slice(digest2.digest).ok();
                command!(self.se.run_command(&DeleteCryptoObj {
                    id: digest_id,
                }, &mut buf2); "Deleting digest object");
            }
            0xE0 => {
                let mut buf = [0; 100];
                let key_id = ObjectId(hex!("03445566"));
                command!(self.se.run_command(&DeleteSecureObject { object_id: key_id }, &mut buf); "deleting");
            }
            _ => {}
        }

        debug_now!("Reply length: {}", reply.len());
        Ok(())
    }
    fn reset(&mut self) {}
}

impl<Twi: I2CForT1, D: DelayUs<u32>> iso7816::App for Card<Twi, D> {
    fn aid(&self) -> iso7816::Aid {
        // PIV AID for easier selection
        iso7816::Aid::new_truncatable(&hex!("D2760001FF 01 0304 000F 00000000 0000"), 5)
    }
}

impl<Twi, D, const C: usize, const R: usize> apdu_dispatch::App<C, R> for Card<Twi, D>
where
    Twi: I2CForT1,
    D: DelayUs<u32>,
{
    fn select(
        &mut self,
        command: &iso7816::Command<C>,
        reply: &mut heapless::Vec<u8, R>,
    ) -> Result<(), Status> {
        self.handle(command, reply)
    }

    fn call(
        &mut self,
        _interface: apdu_dispatch::dispatch::Interface,
        command: &iso7816::Command<C>,
        reply: &mut heapless::Vec<u8, R>,
    ) -> Result<(), Status> {
        self.handle(command, reply)
    }

    fn deselect(&mut self) {
        self.reset()
    }
}
