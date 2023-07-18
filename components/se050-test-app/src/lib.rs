#![no_std]

use embedded_hal::blocking::delay::DelayUs;
use hex_literal::hex;
use iso7816::{Instruction, Status};
use se050::{
    se050::{
        commands::{
            CreateSession, DeleteAll, DeleteSecureObject, EcdsaSign, EcdsaVerify, EddsaSign,
            EddsaVerify, GetRandom, ReadEcCurveList, ReadIdList, ReadObject, VerifySessionUserId,
            WriteBinary, WriteEcKey, WriteUserId,
        },
        policies::{ObjectAccessRule, ObjectPolicyFlags, Policy, PolicySet},
        EcCurve, EcDsaSignatureAlgo, ObjectId, P1KeyType, ProcessSessionCmd, Se050, Se050Result,
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
