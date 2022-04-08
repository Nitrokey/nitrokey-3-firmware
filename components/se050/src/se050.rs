use crate::types::*;
use core::convert::{TryFrom, Into};
use embedded_hal::blocking::delay::DelayMs;

#[derive(Debug, PartialEq, Eq)]
pub enum Se050Error {
	UnknownError
}

pub const APDU_INSTRUCTION_TRANSIENT: u8 = 0x80;
pub const APDU_INSTRUCTION_AUTH_OBJECT: u8 = 0x40;
pub const APDU_INSTRUCTION_ATTEST: u8 = 0x20;

#[allow(dead_code)]
#[repr(u8)]
pub enum Se050ApduInstruction { /* mask:0x1f */
	Write = 0x01,
	Read = 0x02,
	Crypto = 0x03,
	Mgmt = 0x04,
	Process = 0x05,
	ImportExternal = 0x06
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Se050ApduP1KeyType { /* mask:0x60 */
	KeyPair = 0x60,
	PrivateKey = 0x40,
	PublicKey = 0x20,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Se050ApduP1CredType {
	Default = 0x00, 
	EC = 0x01,
	RSA = 0x02,
	AES = 0x03,
	DES = 0x04,
	HMAC = 0x05,
	Binary = 0x06,
	UserID = 0x07,
	Counter = 0x08,
	PCR = 0x09,
	Curve = 0x0b,
	Signature = 0x0c,
	MAC = 0x0d, 
	Cipher = 0x0e,
	TLS = 0x0f,
	CryptoObj = 0x10
}       

#[allow(dead_code,non_camel_case_types)]
#[repr(u8)]
pub enum Se050ApduP2 {
	Default = 0x00,
	Generate = 0x03,
	Create = 0x04,
	Size = 0x07,
	Sign = 0x09,
	Verify = 0x0a,
	Init = 0x0b,
	Update = 0x0c,
	Final = 0x0d,
	Oneshot = 0x0e,
	DH = 0x0f,
	Diversify = 0x10,
	AuthFirstPart2 = 0x12,
	AuthNonfirstPart2 = 0x13,
	DumpKey = 0x14,
	ChangeKeyPart1 = 0x15,
	ChangeKeyPart2 = 0x16,
	KillAuth = 0x17, 
	Import = 0x18,
	Export = 0x19,
	SessionCreate = 0x1b,
	SessionClose = 0x1c,
	SessionRefresh = 0x1e,
	SessionPolicy = 0x1f,
	Version = 0x20, 
	Memory = 0x22,
	List = 0x25,
	Type = 0x26,
	Exist = 0x27,
	DeleteObject = 0x28,
	DeleteAll = 0x2a,
	SessionUserID = 0x2c,
	HKDF = 0x2d,
	PBKDF = 0x2e,
	I2CM = 0x30,
	I2CMAttested = 0x31,
	MAC = 0x32,
	UnlockChallenge = 0x33,
	CurveList = 0x34, 
	SignECDAA = 0x35, 
	ID = 0x36,
	EncryptOneshot = 0x37,
	DecryptOneshot = 0x38,
	Attest = 0x3a, 
	Attributes = 0x3b,
	CPLC = 0x3c, 
	Time = 0x3d, 
	Transport = 0x3e,
	Variant = 0x3f,
	Param = 0x40,
	DeleteCurve = 0x41,
	Encrypt = 0x42,
	Decrypt = 0x43,
	Validate = 0x44,
	GenerateOneshot = 0x45,
	ValidateOneshot = 0x46,
	CryptoList = 0x47,
	Random = 0x49,
	TLS_PMS = 0x4a,
	TLS_PRF_CLI_Hello = 0x4b,
	TLS_PRF_SRV_Hello = 0x4c,
	TLS_PRF_CLI_RND = 0x4d,
	TLS_PRF_SRV_RND = 0x4e,
	RAW = 0x4f,
	ImportExt = 0x51,
	SCP = 0x52,
	AuthFirstPart1 = 0x53,
	AuthNonfirstPart1 = 0x54,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Se050ApduSecObjType {
	ECKeyPair = 0x01,
	ECPrivKey = 0x02,
	ECPubKey = 0x03,
	RSAKeyPair = 0x04,
	RSAKeyPairCRT = 0x05,
	RSAPrivKey = 0x06,
	RSAPrivKeyCRT = 0x07,
	RSAPubKey = 0x08,
	AESKey = 0x09,
	DESKey = 0x0a,
	BinaryFile = 0x0b,
	UserID = 0x0c,
	Counter = 0x0d,
	PCR = 0x0f,
	Curve = 0x10,
	HMACKey = 0x11,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Se050ApduMemoryType {
	Persistent = 1,
	TransientReset = 2,
	TransientDeselect = 3,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Se050ApduObjectOrigin {
	External = 1,
	Internal = 2,
	Provisioned = 3
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Se050TlvTag {
	SessionID = 0x10,
	Policy = 0x11,
	MaxAttempts = 0x12,
	ImportAuthData = 0x13,
	ImportAuthKeyID = 0x14,
	Tag1 = 0x41,
	Tag2 = 0x42,
	Tag3 = 0x43,
	Tag4 = 0x44,
	Tag5 = 0x45,
	Tag6 = 0x46,
	Tag7 = 0x47,
	Tag8 = 0x48,
	Tag9 = 0x49,
	Tag10 = 0x4a,
}

include!("se050_convs.rs");

//////////////////////////////////////////////////////////////////////////////

pub trait Se050Device {
	fn enable(&mut self, delay: &mut impl DelayMs<u32>) -> Result<(), Se050Error>;
	fn disable(&mut self, delay: &mut impl DelayMs<u32>);
	fn get_random(&mut self, buf: &mut [u8], delay: &mut impl DelayMs<u32>) -> Result<(), Se050Error>;
	fn write_aes_key(&mut self, key: &[u8], delay: &mut impl DelayMs<u32>) -> Result<(), Se050Error>;
	fn encrypt_aes_oneshot(&mut self, data: &[u8], enc: &mut [u8], delay: &mut impl DelayMs<u32>) -> Result<(), Se050Error>;
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Se050AppInfo {
	applet_version: u32,
	features: u16,
	securebox_version: u16
}

pub struct Se050<T> where T: T1Proto {
	t1_proto: T,
	atr_info: Option<AnswerToReset>,
	app_info: Option<Se050AppInfo>,
}

impl<T> Se050<T> where T: T1Proto {

	pub fn new(t1: T) -> Se050<T> {
		Se050 {
			t1_proto: t1,
			atr_info: None,
			app_info: None,
		}
	}

}

impl<T> Se050Device for Se050<T> where T: T1Proto {

	fn enable(&mut self, delay: &mut impl DelayMs<u32>) -> Result<(), Se050Error> {
		/* Step 1: perform interface soft reset, parse ATR */
		let r = self.t1_proto.interface_soft_reset(delay);
		if r.is_err() {
			error!("SE050 Interface Reset Error");
			return Err(Se050Error::UnknownError);
		}
		self.atr_info = r.ok();
		debug!("SE050 ATR: {:?}", self.atr_info.as_ref().unwrap());

		/* Step 2: send GP SELECT to choose SE050 JCOP APP, parse APP version */
		let app_id: [u8; 16] = [0xA0, 0x00, 0x00, 0x03, 0x96, 0x54, 0x53, 0x00,
                        		0x00, 0x00, 0x01, 0x03, 0x00, 0x00, 0x00, 0x00];
		let app_select_apdu = CApdu::new(
			ApduClass::StandardPlain,
			ApduStandardInstruction::SelectFile.into(),
			0x04, 0x00, &app_id);
		self.t1_proto.send_apdu(&app_select_apdu, 0, delay).map_err(|_| Se050Error::UnknownError)?;

		let mut appid_data: [u8; 11] = [0; 11];
		let mut appid_apdu = RApdu::blank();
		self.t1_proto.receive_apdu(&mut appid_data, &mut appid_apdu, delay).map_err(|_| Se050Error::UnknownError)?;

		let adata = appid_apdu.data;
		let asw = appid_apdu.sw;
		if asw != 0x9000 || adata.len() != 7 {
			error!("SE050 GP SELECT Err: {:?} {:x}", hexstr!(adata), asw);
			return Err(Se050Error::UnknownError);
		}

		self.app_info = Some(Se050AppInfo {
				applet_version: get_u24_be(&adata[0..3]),
				features: get_u16_be(&adata[3..5]),
				securebox_version: get_u16_be(&adata[5..7])
		});
		debug!("SE050 App: {:?}", self.app_info.as_ref().unwrap());

		Ok(())
	}

	fn disable(&mut self, delay: &mut impl DelayMs<u32>) {
		// send S:EndApduSession
		// receive ACK
		// power down
	}

	#[inline(never)]
	fn get_random(&mut self, buf: &mut [u8], delay: &mut impl DelayMs<u32>) -> Result<(), Se050Error> {
		if buf.len() > 250 { todo!(); }
		let tlv1: [u8; 4] = [Se050TlvTag::Tag1.into(), 0x02, 0x00, buf.len() as u8];
		let capdu = CApdu::new(
			ApduClass::ProprietaryPlain,
			Se050ApduInstruction::Mgmt.into(),
			Se050ApduP1CredType::Default.into(),
			Se050ApduP2::Random.into(),
			&tlv1);
		self.t1_proto.send_apdu(&capdu, 0, delay).map_err(|_| Se050Error::UnknownError)?;

		let mut rapdu_buf: [u8; 260] = [0; 260];
		let mut rapdu = RApdu::blank();
		self.t1_proto.receive_apdu(&mut rapdu_buf, &mut rapdu, delay).map_err(|_| Se050Error::UnknownError)?;

		if rapdu.sw != 0x9000 || rapdu.data[0] != Se050TlvTag::Tag1.into() {
			error!("SE050 GetRandom Failed: {:x}", rapdu.sw);
			return Err(Se050Error::UnknownError);
		}

		if rapdu.data[1] == 0x82 {
			let rcvlen = get_u16_be(&rapdu.data[2..4]) as usize;
			if rcvlen != buf.len() {
				error!("SE050 GetRandom Length Mismatch");
				return Err(Se050Error::UnknownError);
			}
			for i in 0..rcvlen {
				buf[i] = rapdu.data[4+i];
			}
			Ok(())
		} else if rapdu.data[1] < 0x80 {
			let rcvlen: usize = rapdu.data[1] as usize;
			if rcvlen != buf.len() {
				error!("SE050 GetRandom Length Mismatch");
				return Err(Se050Error::UnknownError);
			}
			for i in 0..rcvlen {
				buf[i] = rapdu.data[2+i];
			}
			Ok(())
		} else {
			error!("SE050 GetRandom Invalid R-APDU Length");
			Err(Se050Error::UnknownError)
		}
	}

	#[inline(never)]
	/* no support yet for rfc3394 key wrappings, policies or max attempts */
	fn write_aes_key(&mut self, key: &[u8], delay: &mut impl DelayMs<u32>) -> Result<(), Se050Error> {
		if key.len() != 16 { todo!(); }
		let mut tlvs: [u8; 2+4+2+16] = [Se050TlvTag::Tag1.into(), 0x04,
					/* ObjID */ 0xae,0x50,0xae,0x50,
					Se050TlvTag::Tag3.into(), 0x10,
					/* key data */ 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
		tlvs[8..].clone_from_slice(key);
		let capdu = CApdu::new(
			ApduClass::ProprietaryPlain,
			Into::<u8>::into(Se050ApduInstruction::Write) | APDU_INSTRUCTION_TRANSIENT,
			Se050ApduP1CredType::AES.into(),
			Se050ApduP2::Default.into(),
			&tlvs);
		self.t1_proto.send_apdu(&capdu, 0, delay).map_err(|_| Se050Error::UnknownError)?;

		let mut rapdu_buf: [u8; 260] = [0; 260];
		let mut rapdu = RApdu::blank();
		self.t1_proto.receive_apdu(&mut rapdu_buf, &mut rapdu, delay).map_err(|_| Se050Error::UnknownError)?;

		if rapdu.sw != 0x9000 {
			error!("SE050 WriteAESKey Failed: {:x}", rapdu.sw);
			return Err(Se050Error::UnknownError);
		}

		Ok(())
	}

	#[inline(never)]
	/* hardcoded ObjID */
	fn encrypt_aes_oneshot(&mut self, data: &[u8], enc: &mut [u8], delay: &mut impl DelayMs<u32>) -> Result<(), Se050Error> {
		if data.len() > 240 || (data.len() % 16 != 0) {
			error!("Input data too long or unaligned");
			return Err(Se050Error::UnknownError);
		}
		if enc.len() != data.len() {
			error!("Insufficient output buffer");
			return Err(Se050Error::UnknownError);
		}
		let mut tlvs: [u8; 254] = [0; 254];
		tlvs[0..11].clone_from_slice(&[Se050TlvTag::Tag1.into(), 0x04,
					/* ObjID */ 0xae,0x50,0xae,0x50,
					Se050TlvTag::Tag2.into(), 0x01,
					/* CipherMode */ 0x0d /* AES CBC NOPAD */,
					Se050TlvTag::Tag3.into(), data.len() as u8]);
		tlvs[11..(11+data.len())].clone_from_slice(data);
		let capdu = CApdu::new(
			ApduClass::ProprietaryPlain,
			Se050ApduInstruction::Crypto.into(),
			Se050ApduP1CredType::Cipher.into(),
			Se050ApduP2::EncryptOneshot.into(),
			&tlvs[0..(data.len()+11)]);
		self.t1_proto.send_apdu(&capdu, 0, delay).map_err(|_| Se050Error::UnknownError)?;

		let mut rapdu_buf: [u8; 260] = [0; 260];
		let mut rapdu = RApdu::blank();
		self.t1_proto.receive_apdu(&mut rapdu_buf, &mut rapdu, delay).map_err(|_| Se050Error::UnknownError)?;

		if rapdu.sw != 0x9000 || rapdu.data[0] != Se050TlvTag::Tag1.into() {
			error!("SE050 EncryptAESOneshot Failed: {:x}", rapdu.sw);
			return Err(Se050Error::UnknownError);
		}

		if rapdu.data[1] == 0x82 {
			let rcvlen = get_u16_be(&rapdu.data[2..4]) as usize;
			if rcvlen != enc.len() {
				error!("SE050 EncryptAESOneshot Length Mismatch");
				return Err(Se050Error::UnknownError);
			}
			for i in 0..rcvlen {
				enc[i] = rapdu.data[4+i];
			}
			Ok(())
		} else if rapdu.data[1] < 0x80 {
			let rcvlen: usize = rapdu.data[1] as usize;
			if rcvlen != enc.len() {
				error!("SE050 EncryptAESOneshot Length Mismatch");
				return Err(Se050Error::UnknownError);
			}
			for i in 0..rcvlen {
				enc[i] = rapdu.data[2+i];
			}
			Ok(())
		} else {
			error!("SE050 EncryptAESOneshot Invalid R-APDU Length");
			Err(Se050Error::UnknownError)
		}
	}
}
