use core::convert::{TryFrom, Into};

pub enum Iso7816Error {
	ValueError
}

#[derive(Copy,Clone)]
#[allow(dead_code)]
#[repr(u8)]
pub enum ApduClass {
	StandardPlain = 0b0000_0000,
	ProprietaryPlain = 0b1000_0000,
	ProprietarySecure = 0b1000_0100,
}

#[derive(Copy,Clone)]
#[allow(dead_code)]
#[repr(u8)]
pub enum ApduStandardInstruction {
	EraseBinary = 0x0e,
	Verify = 0x20,
	ManageChannel = 0x70,
	ExternalAuthenticate = 0x82,
	GetChallenge = 0x84,
	InternalAuthenticate = 0x88,
	SelectFile = 0xa4,
	ReadBinary = 0xb0,
	ReadRecords = 0xb2,
	GetResponse = 0xc0,
	Envelope = 0xc2,
	GetData = 0xca,
	WriteBinary = 0xd0,
	WriteRecord = 0xd2,
	UpdateBinary = 0xd6,
	PutData = 0xda,
	UpdateData = 0xdc,
	AppendRecord = 0xe2
}

//////////////////////////////////////////////////////////////////////////////

pub struct SimpleTlv<'a> {
	tag: u8,
	data: &'a [u8]
}

pub struct CApdu<'a> {
	pub cla: ApduClass,
	pub ins: u8,
	pub p1: u8,
	pub p2: u8,
	pub data: &'a [u8]
}

pub struct RApdu<'a> {
	pub data: &'a [u8],
	pub sw: u16
}

impl<'a> CApdu<'a> {
	pub fn new(cla: ApduClass, ins: u8, p1: u8, p2: u8, data: &'a [u8]) -> Self {
		Self { cla, ins, p1, p2, data }
	}

	pub fn blank() -> Self {
		Self { cla: ApduClass::StandardPlain, ins: 0, p1: 0, p2: 0, data: &[] }
	}
}

impl<'a> RApdu<'a> {
	pub fn blank() -> Self {
		Self { data: &[], sw: 0x0000 }
	}
}

//////////////////////////////////////////////////////////////////////////////

pub const T1_S_REQUEST_CODE: u8 = 0b1100_0000;
pub const T1_S_RESPONSE_CODE: u8 = 0b1110_0000;

pub const T1_R_CODE_MASK: u8 = 0b1110_1100;
pub const T1_R_CODE: u8 = 0b1000_0000;

pub enum T1SCode {
	Resync = 0,
	IFS = 1,
	Abort = 2,
	WTX = 3,
	EndApduSession = 5,
	ChipReset = 6,
	GetATR = 7,
	InterfaceSoftReset = 15
}

pub enum T1Error {
	TransmitError,
	ReceiveError,
	BufferOverrunError(usize),
	ChecksumError,
	ProtocolError,
	RCodeReceived(u8),
}

pub trait T1Proto {
	fn send_apdu(&mut self, apdu: &CApdu, le: u8) -> Result<(), T1Error>;
	fn receive_apdu<'a, 'b>(&mut self, buf: &'b mut [u8], apdu: &'a mut RApdu<'b>) -> Result<(), T1Error>;
	fn interface_soft_reset(&mut self) -> Result<AnswerToReset, T1Error>;
}

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct AnswerToReset {
	pub protocol_version: u8,
	pub vendor_id: [u8; 5],
	// Data Link Layer Parameters
	pub dllp: DataLinkLayerParameters,
	// Physical Layer Parameters
	pub plp: PhysicalLayerParameters,
	// Historical Bytes (truncated to save memory)
	pub historical_bytes: [u8; 15]
}

#[derive(Debug)]
pub struct DataLinkLayerParameters {
	pub bwt_ms: u16,
	pub ifsc: u16,
}

#[derive(Debug)]
pub enum PhysicalLayerParameters {
	I2C(I2CParameters)
}

#[derive(Debug)]
pub struct I2CParameters {
	pub mcf: u16,
	pub configuration: u8,
	pub mpot_ms: u8,
	pub rfu: [u8; 3],
	pub segt_us: u16,
	pub wut_us: u16,
}

//////////////////////////////////////////////////////////////////////////////

const CRC16_CCITT_XORLUT: [u16; 256] = [
	0x0000, 0x1189, 0x2312, 0x329b, 0x4624, 0x57ad, 0x6536, 0x74bf,
	0x8c48, 0x9dc1, 0xaf5a, 0xbed3, 0xca6c, 0xdbe5, 0xe97e, 0xf8f7,
	0x1081, 0x0108, 0x3393, 0x221a, 0x56a5, 0x472c, 0x75b7, 0x643e,
	0x9cc9, 0x8d40, 0xbfdb, 0xae52, 0xdaed, 0xcb64, 0xf9ff, 0xe876,
	0x2102, 0x308b, 0x0210, 0x1399, 0x6726, 0x76af, 0x4434, 0x55bd,
	0xad4a, 0xbcc3, 0x8e58, 0x9fd1, 0xeb6e, 0xfae7, 0xc87c, 0xd9f5,
	0x3183, 0x200a, 0x1291, 0x0318, 0x77a7, 0x662e, 0x54b5, 0x453c,
	0xbdcb, 0xac42, 0x9ed9, 0x8f50, 0xfbef, 0xea66, 0xd8fd, 0xc974,
	0x4204, 0x538d, 0x6116, 0x709f, 0x0420, 0x15a9, 0x2732, 0x36bb,
	0xce4c, 0xdfc5, 0xed5e, 0xfcd7, 0x8868, 0x99e1, 0xab7a, 0xbaf3,
	0x5285, 0x430c, 0x7197, 0x601e, 0x14a1, 0x0528, 0x37b3, 0x263a,
	0xdecd, 0xcf44, 0xfddf, 0xec56, 0x98e9, 0x8960, 0xbbfb, 0xaa72,
	0x6306, 0x728f, 0x4014, 0x519d, 0x2522, 0x34ab, 0x0630, 0x17b9,
	0xef4e, 0xfec7, 0xcc5c, 0xddd5, 0xa96a, 0xb8e3, 0x8a78, 0x9bf1,
	0x7387, 0x620e, 0x5095, 0x411c, 0x35a3, 0x242a, 0x16b1, 0x0738,
	0xffcf, 0xee46, 0xdcdd, 0xcd54, 0xb9eb, 0xa862, 0x9af9, 0x8b70,
	0x8408, 0x9581, 0xa71a, 0xb693, 0xc22c, 0xd3a5, 0xe13e, 0xf0b7,
	0x0840, 0x19c9, 0x2b52, 0x3adb, 0x4e64, 0x5fed, 0x6d76, 0x7cff,
	0x9489, 0x8500, 0xb79b, 0xa612, 0xd2ad, 0xc324, 0xf1bf, 0xe036,
	0x18c1, 0x0948, 0x3bd3, 0x2a5a, 0x5ee5, 0x4f6c, 0x7df7, 0x6c7e,
	0xa50a, 0xb483, 0x8618, 0x9791, 0xe32e, 0xf2a7, 0xc03c, 0xd1b5,
	0x2942, 0x38cb, 0x0a50, 0x1bd9, 0x6f66, 0x7eef, 0x4c74, 0x5dfd,
	0xb58b, 0xa402, 0x9699, 0x8710, 0xf3af, 0xe226, 0xd0bd, 0xc134,
	0x39c3, 0x284a, 0x1ad1, 0x0b58, 0x7fe7, 0x6e6e, 0x5cf5, 0x4d7c,
	0xc60c, 0xd785, 0xe51e, 0xf497, 0x8028, 0x91a1, 0xa33a, 0xb2b3,
	0x4a44, 0x5bcd, 0x6956, 0x78df, 0x0c60, 0x1de9, 0x2f72, 0x3efb,
	0xd68d, 0xc704, 0xf59f, 0xe416, 0x90a9, 0x8120, 0xb3bb, 0xa232,
	0x5ac5, 0x4b4c, 0x79d7, 0x685e, 0x1ce1, 0x0d68, 0x3ff3, 0x2e7a,
	0xe70e, 0xf687, 0xc41c, 0xd595, 0xa12a, 0xb0a3, 0x8238, 0x93b1,
	0x6b46, 0x7acf, 0x4854, 0x59dd, 0x2d62, 0x3ceb, 0x0e70, 0x1ff9,
	0xf78f, 0xe606, 0xd49d, 0xc514, 0xb1ab, 0xa022, 0x92b9, 0x8330,
	0x7bc7, 0x6a4e, 0x58d5, 0x495c, 0x3de3, 0x2c6a, 0x1ef1, 0x0f78,
];

pub fn crc16_ccitt_oneshot(buf: &[u8]) -> u16 {
	let mut crc: u16 = crc16_ccitt_init();
	crc = crc16_ccitt_update(crc, buf);
	crc16_ccitt_final(crc)
}

pub fn crc16_ccitt_init() -> u16 { 0xffff }

pub fn crc16_ccitt_update(mut crc: u16, buf: &[u8]) -> u16 {
	for i in 0..buf.len() {
		let lutbyte: u8 = (crc ^ (buf[i] as u16)) as u8;
		crc = (crc >> 8) ^ CRC16_CCITT_XORLUT[lutbyte as usize];
	}
	crc
}

pub fn crc16_ccitt_final(crc: u16) -> u16 { crc ^ 0xffff }

pub fn get_u16_le(buf: &[u8]) -> u16 {
	(buf[0] as u16) | ((buf[1] as u16) << 8)
}

pub fn set_u16_le(buf: &mut [u8], crc: u16) {
	buf[0] = crc as u8;
	buf[1] = (crc >> 8) as u8;
}

pub fn get_u16_be(buf: &[u8]) -> u16 {
	(buf[1] as u16) | ((buf[0] as u16) << 8)
}

pub fn set_u16_be(buf: &mut [u8], crc: u16) {
	buf[1] = crc as u8;
	buf[0] = (crc >> 8) as u8;
}

pub fn get_u24_be(buf: &[u8]) -> u32 {
	(buf[2] as u32) | ((buf[1] as u32) << 8) | ((buf[0] as u32) << 16)
}

//////////////////////////////////////////////////////////////////////////////

include!("types_convs.rs");
