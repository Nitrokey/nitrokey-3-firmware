use core::convert::TryInto;
use nrf52840_hal::{
	gpio::{Input, Output, Pin, PullDown, PushPull},
	prelude::{OutputPin},
	uarte::Uarte,
};

//////////////////////////////////////////////////////////////////////////////
// PROTOCOL DEFINITIONS

const FPR_MAGIC: u16 = 0xef01;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum PacketType {
	Command			= 0x01,
	DataIntermediate	= 0x02,
	Response		= 0x07,
	DataLast		= 0x08
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum CommandCode {
	GetImg			=	0x01,	//Getting fingerprint images for verification
	Img2Tz			=	0x02,	//Feature extraction from fingerprint image
	Match			=	0x03,	//Comparing two fingerprint features
	Search			=	0x04,	//Fingerprint identification and comparison within all or specified registered fingerprint feature libraries
	RegModel		=	0x05,	//Combining 2-3 fingerprint features into a fingerprint registration template
	StoreModel		=	0x06,	//Store registration template in FLASH
	LoadChar		=	0x07,	//Read a template from FLASH into the cache
	UpChar			=	0x08,	//Upload the feature template of the buffer to the host computer
	DownChar		=	0x09,	//Download a feature template from the host computer to the buffer
	UpImage			=	0x0A,	//Upload the fingerprint image of the buffer to the host computer
	DeleteChar		=	0x0C,	//Delete a feature from FLASH
	Empty			=	0x0D,	//Clear FLASH Fingerprint Database
	SetSysPara		=	0x0E,	//Set Module Parameters
	ReadSysPara		=	0x0F,	//Read Module Parameters
	SetPwd			=	0x12,	//Set Module Password
	VfyPwd			=	0x13,	//Verify Module Password
	SetAddr			=	0x15,	//Set Module Address
	ReadINFPage		=	0x16,	//Read information page content
	WriteNotePad		=	0x18,	//Write a 32-byte Notepad
	ReadNotePad		=	0x19,	//Read a 32-byte Notepad
	HISearch		=	0x1b,	//Search and identify quickly
	TemplateNum		=	0x1D,	//Read the number of templates in the database
	ReadConList		=	0x1F,	//Read available tags of templates in the database
	Cancel			=	0x30,	//Cancel instruction
	AutoEnrol		=	0x31,	//Automatic fingerprint Enrollment
	AutoIdentify		=	0x32,	//Automatic fingerprint Indentification
	GetMinEmptyID		=	0xA0,	//Get the minimum empty ID
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum ResponseCode {
	RetOK			=	0x00,	//success
	RetInvalidPacket	=	0x01,	//Invalid Packet
	RetNoFinger		=	0x02,	//Sensor did not detect finger
	RetStoreImageFail	=	0x03,	//Failed to save image in Image Buffer
	RetTooLowQuality	=	0x06,	//Image quality is too poor to extract features
	RetTooFewPoint		=	0x07,	//Too few feature points to extract features
	RetNotMatched		=	0x08,	//Inconsistent fingerprint template matching
	RetNotIdentified	=	0x09,	//No matching fingerprints
	RetMergeFail		=	0x0A,	//Merge feature failure
	RetInvalidTempID	=	0x0B,	//Invalid template ID
	RetReadTempFail		=	0x0C,	//Failed to read template from database
	RetUpTempFail		=	0x0D,	//Failed to upload template
	RetModBusyErr		=	0x0E,	//The module is busy to receive the data packet now
	RetUpImgFail		=	0x0F,	//Failure to upload image
	RetRemoveTempFail	=	0x10,	//Failed to delete template from database
	RetRemoveAllFail	=	0x11,	//Failed to delete all templates from the database
	RetInvalidPwd		=	0x13,	//Invalid password
	RetInvalidImg		=	0x15,	//There is no valid image data in Image Buffer
	RetLatentFP		=	0x17,	//Latent Fingerprint
	RetDBFull		=	0x1F,	//Database full
	RetInvalidMAddr		=	0x20,	//Illegal module address
	RetNeedVfyPwd		=	0x21,	//The password needs to be verified
	RetIDDuplicate		=	0x22,	//There are duplicate IDs
	RetTemplateEmpty	=	0x23,	//Template is empty
	RetDBBitEmpty		=	0x24,	//Database Bit is empty
	RetInvalidFeatureNum	=	0x25,	//Invalid number of features
	RetTimeout		=	0x26,	//Timeout
	RetFPDuplicate		=	0x27,	//There are duplicate Fingerprints in DB
	RetBusy			=	0x37,	//Busy
}

#[repr(u8)]
#[allow(dead_code)]
enum Baudrate {
	B9600	= 1,
	B19200	= 2,
	B28800	= 3,
	B38400	= 4,
	B48000	= 5,
	B57600	= 6,
	B67200	= 7,
	B76800	= 8,
	B86400	= 9,
	B96000	= 10,
	B105600	= 11,
	B115200	= 12,
}

#[repr(u8)]
#[allow(dead_code)]
enum PacketSize {
	S32	= 0,
	S64	= 1,
	S128	= 2,
	S256	= 3,
}

fn checksum(b: &[u8], iv: u16) -> u16 {
	let mut chksum: u16 = iv;

	for i in 0..b.len() {
		chksum += b[i] as u16;
	}

	chksum
}

//////////////////////////////////////////////////////////////////////////////

pub enum FPRError {
	InitFailed,
	ReadError,
	WriteError,
	BufferOverrun,
	HeaderError,
	ChecksumError,
	PacketParseError,
	UnknownError
}

pub struct FingerprintReader<T> {
	uart: Uarte<T>,
	power_pin: Pin<Output<PushPull>>,
	detect_pin: Pin<Input<PullDown>>,
	packet_header: [u8; 6]
}

impl<T> FingerprintReader<T> where T: nrf52840_hal::uarte::Instance {

	pub fn new(uart: Uarte<T>, addr: u32, pwr_pin: Pin<Output<PushPull>>, det_pin: Pin<Input<PullDown>>) -> Self {
		Self { uart, power_pin: pwr_pin, detect_pin: det_pin,
			packet_header: [(FPR_MAGIC >> 8) as u8, FPR_MAGIC as u8,
			 (addr >> 24) as u8, (addr >> 16) as u8, (addr >> 8) as u8, addr as u8] }
	}

	pub fn power_up(&mut self) -> Result<(), FPRError> {
		self.power_pin.set_low().ok();
		let mut resp: [u8; 2] = [0, 0];

		debug!("FPR: on, awaiting ready");
		self.uart.read(&mut resp[0..1]).ok();
		if resp[0] != 0x55 {
			return Err(FPRError::InitFailed);
		}

		let ssp_plc: [u8; 3] = [CommandCode::SetSysPara as u8, 6, PacketSize::S256 as u8];
		self.command(&ssp_plc, &mut resp)?;
		debug!("FPR: SSP>PLC response {:02x}", resp[0]);
		let ssp_sec: [u8; 3] = [CommandCode::SetSysPara as u8, 5, 5];
		self.command(&ssp_sec, &mut resp)?;
		debug!("FPR: SSP>SEC response {:02x}", resp[0]);

		Ok(())
	}

	pub fn is_enrolled(&mut self) -> bool {
		let mut resp: [u8; 3] = [0; 3];
		let gnm: [u8; 1] = [CommandCode::TemplateNum as u8];
		if self.command(&gnm, &mut resp).is_err() {
			warn!("FPR: error getting template count");
			return false;
		}
		let count: u16 = u16::from_be_bytes(resp[1..3].try_into().unwrap());
		info!("FPR: has {} templates", count);

		count != 0
	}

	pub fn power_down(&mut self) -> Result<(), FPRError> {
		debug!("FPR: off");
		self.power_pin.set_high().ok();
		Ok(())
	}

	pub fn check_detect(&self, latches: &[u32]) -> bool {
		crate::types::is_pin_latched(&self.detect_pin, latches)
	}

	pub fn enrol(&mut self) -> Result<(), FPRError> {
		let fpr_id: u16 = 0x001f;
		let cc: u16 = 0b0000_0000_0010_1010;	/* NoMoveAway, Overwrite, AckEachStep, OpenPretreatment */

		let mut cmd: [u8; 6] = [0; 6];
		cmd[0] = CommandCode::AutoEnrol as u8;
		cmd[1..3].copy_from_slice(&fpr_id.to_be_bytes());
		cmd[3] = 3;
		cmd[4..6].copy_from_slice(&cc.to_be_bytes());

		info!("FPR: enrol cmd");
		self.send(&cmd)?;

		loop {
			let mut rsp: [u8; 8] = [0; 8];
			self.receive(&mut rsp)?;
			info!("FPR: enrol status {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}", rsp[0], rsp[1], rsp[2], rsp[3], rsp[4], rsp[5]);
			if rsp[0] != ResponseCode::RetOK as u8 {
				return Err(FPRError::UnknownError);
			} else if rsp[1] == 6 {
				break;
			}
		}

		Ok(())
	}

	pub fn verify(&mut self) -> Result<bool, FPRError> {
		let cc: u16 = 0b0000_0000_0000_0010;	/* AckEachStep, OpenPretreatment */

		let mut cmd: [u8; 6] = [0; 6];
		cmd[0] = CommandCode::AutoIdentify as u8;
		cmd[1] = 5;
		cmd[2] = 0xff;
		cmd[3] = 0xff;
		cmd[4..6].copy_from_slice(&cc.to_be_bytes());

		info!("FPR: verify cmd");
		self.send(&cmd)?;

		loop {
			let mut rsp: [u8; 8] = [0; 8];
			self.receive(&mut rsp)?;
			info!("FPR: verify status {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}", rsp[0], rsp[1], rsp[2], rsp[3], rsp[4], rsp[5]);
			if rsp[0] != ResponseCode::RetOK as u8 {
				if rsp[0] == 0x09 && rsp[1] == 5 {
					return Ok(false)
				}
				return Err(FPRError::UnknownError);
			} else if rsp[1] == 5 {
				break;
			}
		}

		Ok(true)
	}

	pub fn erase(&mut self) -> Result<(), FPRError> {
		let c: [u8; 1] = [CommandCode::Empty as u8];
		let mut r: [u8; 1] = [0];

		self.command(&c, &mut r)
	}

	#[inline(never)]
	fn send(&mut self, cmd: &[u8]) -> Result<(), FPRError> {
		let mut cmdbuf: [u8; 64] = [0; 64];

		let clen = cmd.len() + 2;
		if 9+clen > 64 {
			return Err(FPRError::BufferOverrun);
		}

		cmdbuf[0..6].copy_from_slice(&self.packet_header);
		cmdbuf[6] = PacketType::Command as u8;
		cmdbuf[7..9].copy_from_slice(&(clen as u16).to_be_bytes());
		cmdbuf[9..9+clen-2].copy_from_slice(cmd);
		let chk: u16 = checksum(&cmdbuf[6..9+clen-2], 0);
		cmdbuf[9+clen-2..9+clen].copy_from_slice(&chk.to_be_bytes());

		self.uart.write(&cmdbuf[0..(9+clen)]).map_err(|_| FPRError::WriteError)?;
		Ok(())
	}

	#[inline(never)]
	fn receive(&mut self, resp: &mut [u8]) -> Result<(), FPRError> {
		let mut rsphdr: [u8; 9] = [0; 9];
		let mut rspbuf: [u8; 256] = [0; 256];

		self.uart.read(&mut rsphdr).map_err(|_| FPRError::ReadError)?;

		for i in 0..6 {
			if rsphdr[i] != self.packet_header[i] {
				return Err(FPRError::HeaderError);
			}
		}

		if rsphdr[6] != PacketType::Response as u8 {
			error!("Unsupported FPR Packet");
			todo!();
		}

		trace!("_fpr rsp {:02x} {:02x}{:02x}", rsphdr[6], rsphdr[7], rsphdr[8]);
		let rsplen: usize = u16::from_be_bytes(rsphdr[7..9].try_into().unwrap()) as usize;
		if rsplen > 256 {
			return Err(FPRError::BufferOverrun);
		}
		self.uart.read(&mut rspbuf[0..rsplen]).map_err(|_| FPRError::ReadError)?;

		let chk_calc = checksum(&rspbuf[0..rsplen-2], checksum(&rsphdr[6..9], 0));
		let chk_packet = u16::from_be_bytes(rspbuf[rsplen-2..rsplen].try_into().unwrap());
		if chk_calc != chk_packet {
			return Err(FPRError::ChecksumError);
		}

		for i in 0..core::cmp::min(resp.len(),rsplen-2) {
			resp[i] = rspbuf[i];
		}

		Ok(())
	}

	fn command(&mut self, cmd: &[u8], resp: &mut [u8]) -> Result<(), FPRError> {
		self.send(cmd)?;
		self.receive(resp)
	}
}
