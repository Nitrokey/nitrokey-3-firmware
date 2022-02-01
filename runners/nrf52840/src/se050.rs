use nrf52840_hal::{
	gpio::{Pin, Output, PushPull},
	prelude::{_embedded_hal_blocking_delay_DelayMs, OutputPin},
	twim::{Error as TwimError, Twim},
};
use asm_delay::bitrate::*;

const I2CS_SE050_ADDRESS: u8 = 0x48;

const T1_NAD_HD2SE: u8 = 0x5a;
const T1_NAD_SE2HD: u8 = 0xa5;

struct Se050ATR {
	blockwait_ms: u16,
	minpoll_ms: u8,
	ifsc: u16,
}

pub struct Se050<T> {
	twi: Twim<T>,
	power_pin: Pin<Output<PushPull>>,
	atr_info: Option<Se050ATR>,
	delay_provider: asm_delay::AsmDelay,
	iseq_snd: u8,
	iseq_rcv: u8,
}

#[allow(dead_code,non_camel_case_types)]
#[repr(u8)]
/* bit (1<<5): 0 in request, 1 in response; in this enum always 0 */
pub enum T1_CODES {
	I_		= 0b0000_0000,	// bit6: N(S), bit5: M
	R_		= 0b1000_0000,	// bit0-1: error, bit4: N(R)
	/* S codes */
	RESYNC		= 0b1100_0000,
	IFS		= 0b1100_0001,
	ABORT		= 0b1100_0010,
	WTX		= 0b1100_0011,
	END_APDU_SESSION= 0b1100_0101,
	CHIP_RESET	= 0b1100_0110,
	GET_ATR		= 0b1100_0111,
	IF_SOFT_RESET	= 0b1100_1111,
	/* S response bit */
	S_RESPONSE	= 0b0010_0000,
}

#[derive(Debug)]
pub enum SeError {
	PinError,
	TransmitError(TwimError),
	ReceiveError(TwimError),
	BufferOverrun(u32),
	ProtocolError,
	ChecksumError,
}

// T=1: NAD PCB LEN INF(*LEN) CRC16
// NAD: HD->SE 0x5a
// NAD: SE->HD 0xa5
// PCB-I: 0b0nm00000
// PCB-R: 0b100n00ee
// PCB-S: 0b11sssssq

// CRC: poly 1021, init direct FFFF, final xor FFFF, rev. input, rev. result
// (CRC16_X_25)

// error response: a5 82 00 da 4f
// correct INTF RESET REQ: 5a cf 00 37 7f
// SE050 INTF RESET RESPONSE: (ATF wrapped in T=1 packet)
// 	a5 ef 23
//		00
//		a0 00 00 03 96				(Application Provider: NXP)
//		04 03 e8 00 fe				(DL: BWT = 1000, IFSC = 254)
//		02					(DL Type: I2C)
//		0b 03 e8 08 01 00 00 00 00 64 00 00	(Phys. L.: Max.Clock = 1000, Conf = RFU3, MPOT = 1, RFU = {0,0,0}, SEGT = 64us, WUT = 0us)
//		0a 4a 43 4f 50 34 20 41 54 50 4f	(Hist.: "JCOP4 ATPO")
//	87 77

macro_rules! u8be16 {
($hi:expr, $lo:expr) => ((($hi as u16) << 8) | ($lo as u16))
}

impl<T> Se050<T> where T: nrf52840_hal::twim::Instance {

	pub fn new(twi: Twim<T>, pwr_pin: Pin<Output<PushPull>>) -> Se050<T> {
		Se050 { twi: twi,
			power_pin: pwr_pin,
			atr_info: None,
			delay_provider: asm_delay::AsmDelay::new(64_u32.mhz()),
			iseq_snd: 0u8,
			iseq_rcv: 0u8 }
	}

	pub fn enable(&mut self) -> Result<(), SeError> {
		let mut atr: [u8; 40] = [0u8; 40];

		trace!("SE050 Up");
		self.power_pin.set_high().map_err(|_| SeError::PinError)?;
		trace!("SE050 REQ");
		self.send_request(T1_CODES::IF_SOFT_RESET as u8, None)?;
		trace!("SE050 RSP");
		self.get_response(T1_CODES::IF_SOFT_RESET as u8 | T1_CODES::S_RESPONSE as u8, Some(&mut atr))?;

		self.atr_info.replace(Se050ATR {
				blockwait_ms: u8be16!(atr[10], atr[11]),
				minpoll_ms: atr[19],
				ifsc: u8be16!(atr[12], atr[13]),
		});
		debug!("SE050 ATR: {} {} {}", self.atr_info.as_ref().unwrap().blockwait_ms, self.atr_info.as_ref().unwrap().minpoll_ms, self.atr_info.as_ref().unwrap().ifsc);

		Ok(())
	}

	pub fn disable(&mut self) {
		self.power_pin.set_low().ok();
	}

	pub fn get_applet_id(&mut self) {
		let mut r: Result<(), SeError>;
		let apdu_select: [u8; 22] = [0x00, 0xa4, 0x04, 0x00, 0x10,
			0xA0, 0x00, 0x00, 0x03, 0x96, 0x54, 0x53, 0x00,
			0x00, 0x00, 0x01, 0x03, 0x00, 0x00, 0x00, 0x00,
			0x00];
		let mut resp_select: [u8; 16] = [0; 16];
		/* expect: (T=1 hdr) A50009 (R-APDU body) 0301016FFF010B (R-APDU trailer) 9000 */
			/* applet version: 3.1.1 */
			/* features: 6FFF = everything, but we're (probably) in FIPS mode */
			/* securebox version: 1.11 */

		r = self.send_apdu(&apdu_select);
		if r.is_err() {
			debug!("SE050 SendApdu -> {:?}", r.err().unwrap());
			return;
		}
		r = self.get_response((T1_CODES::I_ as u8) | (self.iseq_rcv << 6), Some(&mut resp_select));
		if r.is_err() {
			debug!("SE050 RecvApdu -> {:?}", r.err().unwrap());
			return;
		}
		debug!("SE050 GP SELECT: {}", hexstr!(&resp_select));
	}

	/* Intermediate-Level request/response handlers */

	fn send_apdu(&mut self, apdu: &[u8]) -> Result<(), SeError> {
		if apdu.len() > 255 {
			todo!();
		} else {
			let code: u8 = (T1_CODES::I_ as u8) | (self.iseq_snd << 6);
			self.iseq_snd ^= 1;
			self.send_request(code, Some(apdu))
		}
	}

	fn send_request(&mut self, code: u8, buf: Option<&[u8]>) -> Result<(), SeError> {
		let mut txbuf: [u8; 256] = [0u8; 256];
		let inflen: usize = if buf.is_some() { buf.unwrap().len() } else { 0 };

		if 3+inflen+2 > 256 {
			return Err(SeError::BufferOverrun((3+inflen+2) as u32));
		}

		txbuf[0] = T1_NAD_HD2SE;
		txbuf[1] = code;
		txbuf[2] = inflen as u8;
		for i in 0..inflen {
			txbuf[3+i] = buf.unwrap()[i];
		}
		let crc = crc16_ccitt(&txbuf[0..3+inflen]);
		txbuf[3+inflen] = crc as u8;
		txbuf[3+inflen+1] = (crc >> 8) as u8;

		self.send_retry_anack(&txbuf[0..3+inflen+2])
	}

	fn get_response(&mut self, code: u8, buf: Option<&mut [u8]>) -> Result<(), SeError> {
		let mut rxbuf: [u8; 264] = [0u8; 264];

		self.recv_retry_anack(&mut rxbuf[0..3])?;

		if rxbuf[0] != T1_NAD_SE2HD {
			debug!("SE050 unexp. NAD {:02x}", rxbuf[0]);
			return Err(SeError::ProtocolError);
		}
		if rxbuf[1] != code {
			debug!("SE050 unexp. PCB {:02x}", rxbuf[1]);
			return Err(SeError::ProtocolError);
		}

		let rlen: usize = (rxbuf[2] + 2) as usize;
		self.recv_retry_anack(&mut rxbuf[3..3+rlen])?;

		if buf.is_some() {
			let buf_ = buf.unwrap();
			for i in 0..core::cmp::min(3+rlen-2, buf_.len()) {
				buf_[i] = rxbuf[i];
			}

			if buf_.len() < 3+rlen-2 {
				debug!("SE050 buffer overflow {} < {}", buf_.len(), rlen-2);
				return Err(SeError::BufferOverrun(rlen as u32));
			}
		}

		let crc_calc = crc16_ccitt(&rxbuf[0..3+rlen-2]);
		let crc_pkt = u8be16!(rxbuf[3+rlen-1], rxbuf[3+rlen-2]);
		if crc_pkt != crc_calc {
			return Err(SeError::ChecksumError);
		}

		Ok(())
	}

	/* Low-Level TWI send/receive functions with retry in case SE is still busy */

	fn send_retry_anack(&mut self, buf: &[u8]) -> Result<(), SeError> {
		loop {
			self.delay_provider.delay_ms(1u32);
			let err = self.twi.write(I2CS_SE050_ADDRESS, buf);
			match err {
			Ok(_) => { return Ok(()); },
			Err(TwimError::AddressNack) => { },
			Err(e) => { return Err(SeError::TransmitError(e)); }
			}
		}
	}

	fn recv_retry_anack(&mut self, buf: &mut [u8]) -> Result<(), SeError> {
		loop {
			self.delay_provider.delay_ms(1u32);
			let err = self.twi.read(I2CS_SE050_ADDRESS, buf);
			match err {
			Ok(_) => { return Ok(()); },
			Err(TwimError::AddressNack) => { },
			Err(e) => { return Err(SeError::ReceiveError(e)); }
			}
		}
	}
}

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

fn crc16_ccitt(buf: &[u8]) -> u16 {
	let mut chk: u16 = 0xffff;

	for i in 0..buf.len() {
		let lutbyte: u8 = (chk ^ (buf[i] as u16)) as u8;
		chk = (chk >> 8) ^ CRC16_CCITT_XORLUT[lutbyte as usize];
	}

	chk ^ 0xffff
}
