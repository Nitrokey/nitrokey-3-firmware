//! This is an interoperability layer,
//! allowing authenticators to implement
//! only CTAP2.

use core::convert::TryInto;
use cortex_m_semihosting::hprintln;
use crate::bytes::{Bytes, consts};

// pub struct WrongData;

pub const NoError: u16 = 0x9000;

#[repr(u16)]
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Error {
    ConditionsNotSatisfied = 0x6985,
    WrongData = 0x6A80,
    WrongLength = 0x6700,
    ClaNotSupported = 0x6E00,
    InsNotSupported = 0x6D00,
}

#[repr(u8)]
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum ControlByte {
    CheckOnly = 0x07,
    EnforceUserPresenceAndSign = 0x03,
    DontEnforceUserPresenceAndSign = 0x08,
}

impl core::convert::TryFrom<u8> for ControlByte {
    type Error = Error;

    fn try_from(byte: u8) -> Result<ControlByte> {
        match byte {
            0x07 => Ok(ControlByte::CheckOnly),
            0x03 => Ok(ControlByte::EnforceUserPresenceAndSign),
            0x08 => Ok(ControlByte::DontEnforceUserPresenceAndSign),
            _ => Err(Error::WrongData),
        }
    }
}

// impl Into<[u8; 2]> for Error {
//     fn into(self) -> [u8; 2] {
//         (self as u16).to_be_bytes()
//     }
// }

// #[derive(Clone,Debug,Eq,PartialEq)]
pub type Result<T> = core::result::Result<T, Error>;

// impl From<WrongData> for Error {
//     fn from(_: WrongData) -> Error {
//         Error::WrongData
//     }
// }


#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Register {
    client_data_hash: Bytes<consts::U32>,
    app_id_hash: Bytes<consts::U32>,
    max_response: usize,
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Authenticate {
    control_byte: ControlByte,
    client_data_hash: Bytes<consts::U32>,
    app_id_hash: Bytes<consts::U32>,
    key_handle: Bytes<consts::U255>,
    max_response: usize,
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub enum Command {
    Register(Register),
    Authenticate(Authenticate),
    Version,
}

// U2FHID uses extended length encoding
fn parse_apdu_data(mut partial: &[u8]) -> Result<(&[u8], usize)> {
    match partial.len() {
        // Lc = Le = 0
        0 => Ok((&[], 0)),
        1 => Err(Error::WrongLength),
        2 => Err(Error::WrongLength),
        3 => {
            // Lc = 0, skipped
            hprintln!("partial 3").ok();
            let nearly_le = u16::from_be_bytes(partial[1..].try_into().unwrap());
            Ok((&[], match nearly_le {
                0 => 65_536usize,
                non_zero => non_zero as usize,
            }))
        },
        _ => {
            hprintln!("case l, partial[..4] = {:?}", &partial[..4]).ok();

            // first byte is zero
            if partial[0] != 0 {
                return Err(Error::WrongLength);
            }

            partial = &partial[1..];

            // next two bytes are Lc, followed by request of length Lc, the possibly Le
            let lc = u16::from_be_bytes(partial[..2].try_into().unwrap()) as usize;
            hprintln!("lc = {}", lc).ok();
            partial = &partial[2..];

            // request
            if partial.len() < lc {
                return Err(Error::WrongLength);
            }
            let request = &partial[..lc];

            // now for expected length
            partial = &partial[lc..];
            match partial.len() {
                0 => Ok((request, 0)),
                2 => {
                    let nearly_le = u16::from_be_bytes(partial.try_into().unwrap());
                    Ok((request, match nearly_le {
                        0 => 65_536usize,
                        non_zero => non_zero as usize,
                    }))
                }
                _ => Err(Error::WrongLength),
            }
        },
    }
    // (0, 0, partial)
}

// TODO: From<AssertionResponse> for ...
// public key: 0x4 || uncompressed (x,y) of NIST P-256 public key
// TODO: add "

impl core::convert::TryFrom<&[u8]> for Command {
    type Error = Error;
    fn try_from(apdu: &[u8]) -> Result<Command> {
        if apdu.len() < 4 {
            return Err(Error::WrongData);
        }
        let cla = apdu[0];
        let ins = apdu[1];
        let p1 = apdu[2];
        let _p2 = apdu[3];

        if cla != 0 {
            return Err(Error::ClaNotSupported);
        }

        if ins == 0x3 {
            // for some weird historical reason, [0, 3, 0, 0, 0, 0, 0, 0, 0]
            // is valid to send here.
            return Ok(Command::Version);
        };

        // now we can expect extended length encoded APDUs
        let (request, max_response) = parse_apdu_data(&apdu[4..])?;

        match ins {
            // register
            0x1 => {
                if request.len() != 64 {
                    return Err(Error::WrongData);
                }
                Ok(Command::Register(Register {
                    client_data_hash: Bytes::try_from_slice(&request[..32]).unwrap(),
                    app_id_hash: Bytes::try_from_slice(&request[32..]).unwrap(),
                    max_response,
                }))
            },

            // authenticate
            0x2 => {
                let control_byte = ControlByte::try_from(p1)?;
                if request.len() < 65 {
                    return Err(Error::WrongData);
                }
                let key_handle_length = request[64] as usize;
                if request.len() != 65 + key_handle_length {
                    return Err(Error::WrongData);
                }
                Ok(Command::Authenticate(Authenticate {
                    control_byte,
                    client_data_hash: Bytes::try_from_slice(&request[..32]).unwrap(),
                    app_id_hash: Bytes::try_from_slice(&request[32..]).unwrap(),
                    key_handle: Bytes::try_from_slice(&request[65..]).unwrap(),
                    max_response,
                }))
            },

            // 0x3 => {
            //     Ok(Command::Version)
            // }
            _ => Err(Error::InsNotSupported),
        }
    }
}

// #[derive(Clone,Debug,Eq,PartialEq/*,Serialize,Deserialize*/)]
// pub struct U2fRequest<'a> {
//     pub command: U2fCommand,
//     pub data: &'a [u8],
//     pub expected_length: usize,
// }

