use crate::types::*;
use core::convert::{Into, TryInto};
use embedded_hal::blocking::delay::DelayMs;

pub struct T1overI2C<TWI> where
		TWI: embedded_hal::blocking::i2c::Read + embedded_hal::blocking::i2c::Write {
	twi: TWI,
	se_address: u16,
	nad_hd2se: u8,
	nad_se2hd: u8,
	iseq_snd: u8,
	iseq_rcv: u8,
}

const TWI_RETRIES: usize = 128;
const TWI_RETRY_DELAY_MS: u32 = 2;

impl<TWI> T1overI2C<TWI> where
		TWI: embedded_hal::blocking::i2c::Read + embedded_hal::blocking::i2c::Write {
	pub fn new(twi: TWI, address: u16, nad: u8) -> Self {
		let nad_r: u8 = ((nad & 0xf0) >> 4) | ((nad & 0x0f) << 4);
		T1overI2C { twi,
			se_address: address,
			nad_hd2se: nad,
			nad_se2hd: nad_r,
			iseq_snd: 0, iseq_rcv: 0 }
	}

	fn twi_write(&mut self, data: &[u8], delay: &mut impl DelayMs<u32>) -> Result<(), T1Error> {
		for _i in 0..TWI_RETRIES {
			let e = self.twi.write(self.se_address as u8, data);
			if e.is_ok() {
				trace!("t1w ok({})", i);
				return Ok(());
			}
			delay.delay_ms(TWI_RETRY_DELAY_MS);
			// TODO: we should only loop on AddressNack errors
			// but the existing traits don't provide an API for that
		}
		trace!("t1w err");
		return Err(T1Error::TransmitError);
	}

	fn twi_read(&mut self, data: &mut [u8], delay: &mut impl DelayMs<u32>) -> Result<(), T1Error> {
		for _i in 0..TWI_RETRIES {
			let e = self.twi.read(self.se_address as u8, data);
			if e.is_ok() {
				trace!("t1r ok({})", i);
				return Ok(());
			}
			delay.delay_ms(TWI_RETRY_DELAY_MS);
			// TODO: we should only loop on AddressNack errors
			// but the existing traits don't provide an API for that
		}
		trace!("t1r err");
		return Err(T1Error::ReceiveError);
	}

	fn send_s(&mut self, code: T1SCode, data: &[u8], delay: &mut impl DelayMs<u32>) -> Result<(), T1Error> {
		let mut buf: [u8; 260] = [0u8; 260];

		buf[0] = self.nad_hd2se;
		buf[1] = T1_S_REQUEST_CODE | <T1SCode as Into<u8>>::into(code);
		buf[2] = data.len() as u8;
		for i in 0..data.len() {
			buf[3+i] = data[i];
		}
		let crc: u16 = crc16_ccitt_oneshot(&buf[0..3+data.len()]);
		set_u16_le(&mut buf[3+data.len()..3+data.len()+2], crc);

		trace!("T1 W S {}", hexstr!(&buf[0..3+data.len()+2]));
		self.twi_write(&buf[0..3+data.len()+2], delay)
	}

	fn receive_s(&mut self, code: T1SCode, data: &mut [u8], delay: &mut impl DelayMs<u32>) -> Result<(), T1Error> {
		self.twi_read(&mut data[0..3], delay)?;
		trace!("T1 R S H {}", hexstr!(&data[0..3]));
		if data[0] != self.nad_se2hd {
			return Err(T1Error::ProtocolError);
		}
		if data[1] != T1_S_RESPONSE_CODE | <T1SCode as Into<u8>>::into(code) {
			if (data[1] & T1_R_CODE_MASK) == T1_R_CODE {
				return Err(T1Error::RCodeReceived(data[1]));
			}
			return Err(T1Error::ProtocolError);
		}
		let dlen: usize = data[2] as usize;
		let mut crc: u16 = crc16_ccitt_init();
		crc = crc16_ccitt_update(crc, &data[0..3]);

		if dlen+2 > data.len() {
			return Err(T1Error::BufferOverrunError(dlen));
		}

		self.twi_read(&mut data[0..dlen+2], delay)?;
		trace!("T1 R S B {}", hexstr!(&data[0..dlen+2]));
		crc = crc16_ccitt_update(crc, &data[0..dlen]);
		crc = crc16_ccitt_final(crc);

		if crc != get_u16_le(&data[dlen..dlen+2]) {
			return Err(T1Error::ChecksumError);
		}

		Ok(())
	}
}

impl<TWI> T1Proto for T1overI2C<TWI> where
		TWI: embedded_hal::blocking::i2c::Read + embedded_hal::blocking::i2c::Write {

	#[inline(never)]
	fn send_apdu(&mut self, apdu: &CApdu, le: u8, delay: &mut impl DelayMs<u32>) -> Result<(), T1Error> {
		let mut apdubuf: [u8; 260] = [0u8; 260];
		if apdu.data.len() > 248 { todo!(); }
		apdubuf[0] = self.nad_hd2se;
		apdubuf[1] = self.iseq_snd << 6;
		apdubuf[2] = (4 + apdu.data.len() + 2) as u8;
		apdubuf[3] = apdu.cla.into();
		apdubuf[4] = apdu.ins;
		apdubuf[5] = apdu.p1;
		apdubuf[6] = apdu.p2;
		apdubuf[7] = apdu.data.len() as u8;
		for i in 0..apdu.data.len() {
			apdubuf[8+i] = apdu.data[i];
		}
		apdubuf[8+apdu.data.len()] = le;
		let crc = crc16_ccitt_oneshot(&apdubuf[0..8+apdu.data.len()+1]);
		set_u16_le(&mut apdubuf[8+apdu.data.len()+1..8+apdu.data.len()+3], crc);

		self.iseq_snd ^= 1;
		trace!("T1 W I {}", hexstr!(&apdubuf[0..8+apdu.data.len()+3]));
		self.twi_write(&apdubuf[0..8+apdu.data.len()+3], delay)
	}

	#[inline(never)]
	fn receive_apdu<'a, 'b>(&mut self, buf: &'b mut [u8], apdu: &'a mut RApdu<'b>, delay: &mut impl DelayMs<u32>) -> Result<(), T1Error> {
		self.twi_read(&mut buf[0..3], delay)?;
		trace!("T1 R I H {}", hexstr!(&buf[0..3]));
		if buf[0] != self.nad_se2hd {
			return Err(T1Error::ProtocolError);
		}
		if buf[1] != self.iseq_rcv << 6 {
			if buf[1] == T1_S_REQUEST_CODE | <T1SCode as Into<u8>>::into(T1SCode::WTX) {
				// TODO: if found to be S:WTX, directly respond and wait again?
				todo!();
			}
			return Err(T1Error::ProtocolError);
		}
		self.iseq_rcv ^= 1;
		let dlen: usize = buf[2] as usize;
		let mut crc: u16 = crc16_ccitt_init();
		crc = crc16_ccitt_update(crc, &buf[0..3]);

		if dlen+2 > buf.len() {
			return Err(T1Error::BufferOverrunError(dlen));
		}

		self.twi_read(&mut buf[0..dlen+2], delay)?;
		trace!("T1 R I B {}", hexstr!(&buf[0..dlen+2]));
		crc = crc16_ccitt_update(crc, &buf[0..dlen]);
		crc = crc16_ccitt_final(crc);

		if crc != get_u16_le(&buf[dlen..dlen+2]) {
			return Err(T1Error::ChecksumError);
		}

		apdu.data = &buf[0..dlen-2];
		apdu.sw = get_u16_be(&buf[dlen-2..dlen]);

		Ok(())
	}

	#[inline(never)]
	fn interface_soft_reset(&mut self, delay: &mut impl DelayMs<u32>) -> Result<AnswerToReset, T1Error> {
		let mut atrbuf: [u8; 64] = [0u8; 64];
		self.send_s(T1SCode::InterfaceSoftReset, &[], delay)?;
		self.receive_s(T1SCode::InterfaceSoftReset, &mut atrbuf, delay)?;

		let atr_pv = atrbuf[0];
		let dllp_len = atrbuf[6];
		if dllp_len != 4 {
			return Err(T1Error::ProtocolError);
		}
		let plp_type = atrbuf[11];
		let plp_len = atrbuf[12];
		if plp_type != 2 /* I2C */ || plp_len != 11 {
			return Err(T1Error::ProtocolError);
		}
		let _hb_len = atrbuf[24];
		/* TODO: check/use length of historical bytes */
		Ok(AnswerToReset {
			protocol_version: atr_pv,
			vendor_id: atrbuf[1..6].try_into().unwrap(),
			dllp: DataLinkLayerParameters {
				bwt_ms: get_u16_be(&atrbuf[7..9]),
				ifsc: get_u16_be(&atrbuf[9..11])
			},
			plp: PhysicalLayerParameters::I2C(I2CParameters {
				mcf: get_u16_be(&atrbuf[13..15]),
				configuration: atrbuf[15],
				mpot_ms: atrbuf[16],
				rfu: atrbuf[17..20].try_into().unwrap(),
				segt_us: get_u16_be(&atrbuf[20..22]),
				wut_us: get_u16_be(&atrbuf[22..24])
			}),
			historical_bytes: atrbuf[25..40].try_into().unwrap()
		})
	}
}
