impl TryFrom<u8> for Se050ApduInstruction {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value & 0x1f {
		0x01 => Ok(Self::Write),
		0x02 => Ok(Self::Read),
		0x03 => Ok(Self::Crypto),
		0x04 => Ok(Self::Mgmt),
		0x05 => Ok(Self::Process),
		0x06 => Ok(Self::ImportExternal),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for Se050ApduInstruction {
	fn into(self) -> u8 {
		match self {
		Self::Write => 0x01,
		Self::Read => 0x02,
		Self::Crypto => 0x03,
		Self::Mgmt => 0x04,
		Self::Process => 0x05,
		Self::ImportExternal => 0x06,
		}
	}
}
impl TryFrom<u8> for Se050ApduP1KeyType {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value & 0x60 {
		0x20 => Ok(Self::PublicKey),
		0x40 => Ok(Self::PrivateKey),
		0x60 => Ok(Self::KeyPair),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for Se050ApduP1KeyType {
	fn into(self) -> u8 {
		match self {
		Self::PublicKey => 0x20,
		Self::PrivateKey => 0x40,
		Self::KeyPair => 0x60,
		}
	}
}
impl TryFrom<u8> for Se050ApduP1CredType {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
		0x00 => Ok(Self::Default),
		0x01 => Ok(Self::EC),
		0x02 => Ok(Self::RSA),
		0x03 => Ok(Self::AES),
		0x04 => Ok(Self::DES),
		0x05 => Ok(Self::HMAC),
		0x06 => Ok(Self::Binary),
		0x07 => Ok(Self::UserID),
		0x08 => Ok(Self::Counter),
		0x09 => Ok(Self::PCR),
		0x0b => Ok(Self::Curve),
		0x0c => Ok(Self::Signature),
		0x0d => Ok(Self::MAC),
		0x0e => Ok(Self::Cipher),
		0x0f => Ok(Self::TLS),
		0x10 => Ok(Self::CryptoObj),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for Se050ApduP1CredType {
	fn into(self) -> u8 {
		match self {
		Self::Default => 0x00,
		Self::EC => 0x01,
		Self::RSA => 0x02,
		Self::AES => 0x03,
		Self::DES => 0x04,
		Self::HMAC => 0x05,
		Self::Binary => 0x06,
		Self::UserID => 0x07,
		Self::Counter => 0x08,
		Self::PCR => 0x09,
		Self::Curve => 0x0b,
		Self::Signature => 0x0c,
		Self::MAC => 0x0d,
		Self::Cipher => 0x0e,
		Self::TLS => 0x0f,
		Self::CryptoObj => 0x10,
		}
	}
}
impl TryFrom<u8> for Se050ApduP2 {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
		0x00 => Ok(Self::Default),
		0x03 => Ok(Self::Generate),
		0x04 => Ok(Self::Create),
		0x07 => Ok(Self::Size),
		0x09 => Ok(Self::Sign),
		0x0a => Ok(Self::Verify),
		0x0b => Ok(Self::Init),
		0x0c => Ok(Self::Update),
		0x0d => Ok(Self::Final),
		0x0e => Ok(Self::Oneshot),
		0x0f => Ok(Self::DH),
		0x10 => Ok(Self::Diversify),
		0x12 => Ok(Self::AuthFirstPart2),
		0x13 => Ok(Self::AuthNonfirstPart2),
		0x14 => Ok(Self::DumpKey),
		0x15 => Ok(Self::ChangeKeyPart1),
		0x16 => Ok(Self::ChangeKeyPart2),
		0x17 => Ok(Self::KillAuth),
		0x18 => Ok(Self::Import),
		0x19 => Ok(Self::Export),
		0x1b => Ok(Self::SessionCreate),
		0x1c => Ok(Self::SessionClose),
		0x1e => Ok(Self::SessionRefresh),
		0x1f => Ok(Self::SessionPolicy),
		0x20 => Ok(Self::Version),
		0x22 => Ok(Self::Memory),
		0x25 => Ok(Self::List),
		0x26 => Ok(Self::Type),
		0x27 => Ok(Self::Exist),
		0x28 => Ok(Self::DeleteObject),
		0x2a => Ok(Self::DeleteAll),
		0x2c => Ok(Self::SessionUserID),
		0x2d => Ok(Self::HKDF),
		0x2e => Ok(Self::PBKDF),
		0x30 => Ok(Self::I2CM),
		0x31 => Ok(Self::I2CMAttested),
		0x32 => Ok(Self::MAC),
		0x33 => Ok(Self::UnlockChallenge),
		0x34 => Ok(Self::CurveList),
		0x35 => Ok(Self::SignECDAA),
		0x36 => Ok(Self::ID),
		0x37 => Ok(Self::EncryptOneshot),
		0x38 => Ok(Self::DecryptOneshot),
		0x3a => Ok(Self::Attest),
		0x3b => Ok(Self::Attributes),
		0x3c => Ok(Self::CPLC),
		0x3d => Ok(Self::Time),
		0x3e => Ok(Self::Transport),
		0x3f => Ok(Self::Variant),
		0x40 => Ok(Self::Param),
		0x41 => Ok(Self::DeleteCurve),
		0x42 => Ok(Self::Encrypt),
		0x43 => Ok(Self::Decrypt),
		0x44 => Ok(Self::Validate),
		0x45 => Ok(Self::GenerateOneshot),
		0x46 => Ok(Self::ValidateOneshot),
		0x47 => Ok(Self::CryptoList),
		0x49 => Ok(Self::Random),
		0x4a => Ok(Self::TLS_PMS),
		0x4b => Ok(Self::TLS_PRF_CLI_Hello),
		0x4c => Ok(Self::TLS_PRF_SRV_Hello),
		0x4d => Ok(Self::TLS_PRF_CLI_RND),
		0x4e => Ok(Self::TLS_PRF_SRV_RND),
		0x4f => Ok(Self::RAW),
		0x51 => Ok(Self::ImportExt),
		0x52 => Ok(Self::SCP),
		0x53 => Ok(Self::AuthFirstPart1),
		0x54 => Ok(Self::AuthNonfirstPart1),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for Se050ApduP2 {
	fn into(self) -> u8 {
		match self {
		Self::Default => 0x00,
		Self::Generate => 0x03,
		Self::Create => 0x04,
		Self::Size => 0x07,
		Self::Sign => 0x09,
		Self::Verify => 0x0a,
		Self::Init => 0x0b,
		Self::Update => 0x0c,
		Self::Final => 0x0d,
		Self::Oneshot => 0x0e,
		Self::DH => 0x0f,
		Self::Diversify => 0x10,
		Self::AuthFirstPart2 => 0x12,
		Self::AuthNonfirstPart2 => 0x13,
		Self::DumpKey => 0x14,
		Self::ChangeKeyPart1 => 0x15,
		Self::ChangeKeyPart2 => 0x16,
		Self::KillAuth => 0x17,
		Self::Import => 0x18,
		Self::Export => 0x19,
		Self::SessionCreate => 0x1b,
		Self::SessionClose => 0x1c,
		Self::SessionRefresh => 0x1e,
		Self::SessionPolicy => 0x1f,
		Self::Version => 0x20,
		Self::Memory => 0x22,
		Self::List => 0x25,
		Self::Type => 0x26,
		Self::Exist => 0x27,
		Self::DeleteObject => 0x28,
		Self::DeleteAll => 0x2a,
		Self::SessionUserID => 0x2c,
		Self::HKDF => 0x2d,
		Self::PBKDF => 0x2e,
		Self::I2CM => 0x30,
		Self::I2CMAttested => 0x31,
		Self::MAC => 0x32,
		Self::UnlockChallenge => 0x33,
		Self::CurveList => 0x34,
		Self::SignECDAA => 0x35,
		Self::ID => 0x36,
		Self::EncryptOneshot => 0x37,
		Self::DecryptOneshot => 0x38,
		Self::Attest => 0x3a,
		Self::Attributes => 0x3b,
		Self::CPLC => 0x3c,
		Self::Time => 0x3d,
		Self::Transport => 0x3e,
		Self::Variant => 0x3f,
		Self::Param => 0x40,
		Self::DeleteCurve => 0x41,
		Self::Encrypt => 0x42,
		Self::Decrypt => 0x43,
		Self::Validate => 0x44,
		Self::GenerateOneshot => 0x45,
		Self::ValidateOneshot => 0x46,
		Self::CryptoList => 0x47,
		Self::Random => 0x49,
		Self::TLS_PMS => 0x4a,
		Self::TLS_PRF_CLI_Hello => 0x4b,
		Self::TLS_PRF_SRV_Hello => 0x4c,
		Self::TLS_PRF_CLI_RND => 0x4d,
		Self::TLS_PRF_SRV_RND => 0x4e,
		Self::RAW => 0x4f,
		Self::ImportExt => 0x51,
		Self::SCP => 0x52,
		Self::AuthFirstPart1 => 0x53,
		Self::AuthNonfirstPart1 => 0x54,
		}
	}
}
impl TryFrom<u8> for Se050ApduSecObjType {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
		0x01 => Ok(Self::ECKeyPair),
		0x02 => Ok(Self::ECPrivKey),
		0x03 => Ok(Self::ECPubKey),
		0x04 => Ok(Self::RSAKeyPair),
		0x05 => Ok(Self::RSAKeyPairCRT),
		0x06 => Ok(Self::RSAPrivKey),
		0x07 => Ok(Self::RSAPrivKeyCRT),
		0x08 => Ok(Self::RSAPubKey),
		0x09 => Ok(Self::AESKey),
		0x0a => Ok(Self::DESKey),
		0x0b => Ok(Self::BinaryFile),
		0x0c => Ok(Self::UserID),
		0x0d => Ok(Self::Counter),
		0x0f => Ok(Self::PCR),
		0x10 => Ok(Self::Curve),
		0x11 => Ok(Self::HMACKey),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for Se050ApduSecObjType {
	fn into(self) -> u8 {
		match self {
		Self::ECKeyPair => 0x01,
		Self::ECPrivKey => 0x02,
		Self::ECPubKey => 0x03,
		Self::RSAKeyPair => 0x04,
		Self::RSAKeyPairCRT => 0x05,
		Self::RSAPrivKey => 0x06,
		Self::RSAPrivKeyCRT => 0x07,
		Self::RSAPubKey => 0x08,
		Self::AESKey => 0x09,
		Self::DESKey => 0x0a,
		Self::BinaryFile => 0x0b,
		Self::UserID => 0x0c,
		Self::Counter => 0x0d,
		Self::PCR => 0x0f,
		Self::Curve => 0x10,
		Self::HMACKey => 0x11,
		}
	}
}
impl TryFrom<u8> for Se050ApduMemoryType {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
		1 => Ok(Self::Persistent),
		2 => Ok(Self::TransientReset),
		3 => Ok(Self::TransientDeselect),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for Se050ApduMemoryType {
	fn into(self) -> u8 {
		match self {
		Self::Persistent => 1,
		Self::TransientReset => 2,
		Self::TransientDeselect => 3,
		}
	}
}
impl TryFrom<u8> for Se050ApduObjectOrigin {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
		1 => Ok(Self::External),
		2 => Ok(Self::Internal),
		3 => Ok(Self::Provisioned),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for Se050ApduObjectOrigin {
	fn into(self) -> u8 {
		match self {
		Self::External => 1,
		Self::Internal => 2,
		Self::Provisioned => 3,
		}
	}
}
impl TryFrom<u8> for Se050TlvTag {
	type Error = Iso7816Error;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
		0x10 => Ok(Self::SessionID),
		0x11 => Ok(Self::Policy),
		0x12 => Ok(Self::MaxAttempts),
		0x13 => Ok(Self::ImportAuthData),
		0x14 => Ok(Self::ImportAuthKeyID),
		0x41 => Ok(Self::Tag1),
		0x42 => Ok(Self::Tag2),
		0x43 => Ok(Self::Tag3),
		0x44 => Ok(Self::Tag4),
		0x45 => Ok(Self::Tag5),
		0x46 => Ok(Self::Tag6),
		0x47 => Ok(Self::Tag7),
		0x48 => Ok(Self::Tag8),
		0x49 => Ok(Self::Tag9),
		0x4a => Ok(Self::Tag10),
		_ => Err(Self::Error::ValueError)
		}
	}
}
impl Into<u8> for Se050TlvTag {
	fn into(self) -> u8 {
		match self {
		Self::SessionID => 0x10,
		Self::Policy => 0x11,
		Self::MaxAttempts => 0x12,
		Self::ImportAuthData => 0x13,
		Self::ImportAuthKeyID => 0x14,
		Self::Tag1 => 0x41,
		Self::Tag2 => 0x42,
		Self::Tag3 => 0x43,
		Self::Tag4 => 0x44,
		Self::Tag5 => 0x45,
		Self::Tag6 => 0x46,
		Self::Tag7 => 0x47,
		Self::Tag8 => 0x48,
		Self::Tag9 => 0x49,
		Self::Tag10 => 0x4a,
		}
	}
}
