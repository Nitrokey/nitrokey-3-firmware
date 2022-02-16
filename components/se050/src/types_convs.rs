impl TryFrom<u8> for ApduClass {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
		0b0000_0000 => Ok(Self::StandardPlain),
		0b1000_0000 => Ok(Self::ProprietaryPlain),
		0b1000_0100 => Ok(Self::ProprietarySecure),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for ApduClass {
	fn into(self) -> u8 {
		match self {
		Self::StandardPlain => 0b0000_0000,
		Self::ProprietaryPlain => 0b1000_0000,
		Self::ProprietarySecure => 0b1000_0100,
		}
	}
}
impl TryFrom<u8> for ApduStandardInstruction {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
		0x0e => Ok(Self::EraseBinary),
		0x20 => Ok(Self::Verify),
		0x70 => Ok(Self::ManageChannel),
		0x82 => Ok(Self::ExternalAuthenticate),
		0x84 => Ok(Self::GetChallenge),
		0x88 => Ok(Self::InternalAuthenticate),
		0xa4 => Ok(Self::SelectFile),
		0xb0 => Ok(Self::ReadBinary),
		0xb2 => Ok(Self::ReadRecords),
		0xc0 => Ok(Self::GetResponse),
		0xc2 => Ok(Self::Envelope),
		0xca => Ok(Self::GetData),
		0xd0 => Ok(Self::WriteBinary),
		0xd2 => Ok(Self::WriteRecord),
		0xd6 => Ok(Self::UpdateBinary),
		0xda => Ok(Self::PutData),
		0xdc => Ok(Self::UpdateData),
		0xe2 => Ok(Self::AppendRecord),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for ApduStandardInstruction {
	fn into(self) -> u8 {
		match self {
		Self::EraseBinary => 0x0e,
		Self::Verify => 0x20,
		Self::ManageChannel => 0x70,
		Self::ExternalAuthenticate => 0x82,
		Self::GetChallenge => 0x84,
		Self::InternalAuthenticate => 0x88,
		Self::SelectFile => 0xa4,
		Self::ReadBinary => 0xb0,
		Self::ReadRecords => 0xb2,
		Self::GetResponse => 0xc0,
		Self::Envelope => 0xc2,
		Self::GetData => 0xca,
		Self::WriteBinary => 0xd0,
		Self::WriteRecord => 0xd2,
		Self::UpdateBinary => 0xd6,
		Self::PutData => 0xda,
		Self::UpdateData => 0xdc,
		Self::AppendRecord => 0xe2,
		}
	}
}
impl TryFrom<u8> for T1SCode {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
		0 => Ok(Self::Resync),
		1 => Ok(Self::IFS),
		15 => Ok(Self::InterfaceSoftReset),
		2 => Ok(Self::Abort),
		3 => Ok(Self::WTX),
		5 => Ok(Self::EndApduSession),
		6 => Ok(Self::ChipReset),
		7 => Ok(Self::GetATR),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for T1SCode {
	fn into(self) -> u8 {
		match self {
		Self::Resync => 0,
		Self::IFS => 1,
		Self::InterfaceSoftReset => 15,
		Self::Abort => 2,
		Self::WTX => 3,
		Self::EndApduSession => 5,
		Self::ChipReset => 6,
		Self::GetATR => 7,
		}
	}
}
