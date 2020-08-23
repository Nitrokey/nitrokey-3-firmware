/*!
The CTAP protocol is a series of atomic *transactions*, which consist
of a *request* message followed by a *response* message.

Messages may spread over multiple *packets*, starting with
an *initialization* packet, followed by zero or more *continuation* packets.

In the case of multiple clients, the first to get through its initialization
packet in device idle state locks the device for other channels (they will
receive busy errors).

No state is maintained between transactions.
*/

use core::convert::TryInto;
use core::convert::TryFrom;
// use cortex_m_semihosting::hprintln;
use serde::Serialize;
use usb_device::{
    bus::{UsbBus},
    endpoint::{EndpointAddress, EndpointIn, EndpointOut},
    UsbError,
    // Result as UsbResult,
};


use crate::{
    authenticator::{
        Api as AuthenticatorApi,
        Error as AuthenticatorError,
    },
    constants::{
        // 7609
        MESSAGE_SIZE,
        // 64
        PACKET_SIZE,
    },
    types::{
        // AssertionResponse,
        // AuthenticatorInfo,
        GetAssertionParameters,
        MakeCredentialParameters,
        ctap1,
    },
};

/// The actual payload of given length is dealt with separately
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
struct Request {
    channel: u32,
    command: Command,
    length: u16,
}

/// The actual payload of given length is dealt with separately
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
struct Response {
    channel: u32,
    command: Command,
    length: u16,
}

impl Response {
    pub fn from_request_and_size(request: Request, size: usize) -> Self {
        Self {
            channel: request.channel,
            command: request.command,
            length: size as u16,
        }
    }
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
struct MessageState {
    // sequence number of next continuation packet
    next_sequence: u8,
    // number of bytes of message payload transmitted so far
    transmitted: usize,
}

impl Default for MessageState {
    fn default() -> Self {
        Self {
            next_sequence: 0,
            transmitted: PACKET_SIZE - 7,
        }
    }
}

impl MessageState {
    // update state due to receiving a full new continuation packet
    pub fn absorb_packet(&mut self) {
        self.next_sequence += 1;
        self.transmitted += PACKET_SIZE - 5;
    }
}

/// the authenticator API, consisting of "operations"
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Operation {
    MakeCredential,
    GetAssertion,
    GetNextAssertion,
    GetInfo,
    ClientPin,
    Reset,
    // new in v2.1
    BioEnrollment,
    // new in v2.1
    CredentialManagement,
    /// vendors are assigned the range 0x40..=0x7f for custom operations
    Vendor(VendorOperation),
}

impl Into<u8> for Operation {
    fn into(self) -> u8 {
        match self {
            Operation::MakeCredential => 0x01,
            Operation::GetAssertion => 0x02,
            Operation::GetNextAssertion => 0x08,
            Operation::GetInfo => 0x04,
            Operation::ClientPin => 0x06,
            Operation::Reset => 0x07,
            Operation::BioEnrollment => 0x09,
            Operation::CredentialManagement => 0x0A,
            Operation::Vendor(operation) => operation.into(),
        }
    }
}

impl Operation {
    pub fn into_u8(self) -> u8 {
        self.into()
    }
}

/// Vendor CTAP2 operations, from 0x40 to 0x7f.
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub struct VendorOperation(u8);

impl VendorOperation {
    pub const FIRST: u8 = 0x40;
    pub const LAST: u8 = 0x7f;
}

impl TryFrom<u8> for VendorOperation {
    type Error = ();

    fn try_from(from: u8) -> core::result::Result<Self, ()> {
        match from {
            // code if code >= Self::FIRST && code <= Self::LAST => Ok(VendorOperation(code)),
            code @ Self::FIRST..=Self::LAST => Ok(VendorOperation(code)),
            _ => Err(()),
        }
    }
}

impl Into<u8> for VendorOperation {
    fn into(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Operation {
    type Error = ();

    fn try_from(from: u8) -> core::result::Result<Operation, ()> {
        match from {
            0x01 => Ok(Operation::MakeCredential),
            0x02 => Ok(Operation::GetAssertion),
            0x08 => Ok(Operation::GetNextAssertion),
            0x04 => Ok(Operation::GetInfo),
            0x06 => Ok(Operation::ClientPin),
            0x07 => Ok(Operation::Reset),
            0x09 => Ok(Operation::BioEnrollment),
            0x0A => Ok(Operation::CredentialManagement),
            code @ VendorOperation::FIRST..=VendorOperation::LAST
                 => Ok(Operation::Vendor(VendorOperation::try_from(code)?)),
            _ => Err(()),
        }
    }
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum Command {
    // mandatory for CTAP1
    Ping,
    Msg,
    Init,
    Error,

    // optional
    Wink,
    Lock,

    // mandatory for CTAP2
    Cbor,
    Cancel,
    KeepAlive,

    // vendor-assigned range from 0x40 to 0x7f
    Vendor(VendorCommand),
}

impl Command {
    pub fn into_u8(self) -> u8 {
        self.into()
    }
}

impl TryFrom<u8> for Command {
    type Error = ();

    fn try_from(from: u8) -> core::result::Result<Command, ()> {
        match from {
            0x01 => Ok(Command::Ping),
            0x03 => Ok(Command::Msg),
            0x06 => Ok(Command::Init),
            0x3f => Ok(Command::Error),
            0x08 => Ok(Command::Wink),
            0x04 => Ok(Command::Lock),
            0x10 => Ok(Command::Cbor),
            0x11 => Ok(Command::Cancel),
            0x3b => Ok(Command::KeepAlive),
            code => Ok(Command::Vendor(VendorCommand::try_from(code)?)),
        }
    }
}

/// Vendor CTAPHID commands, from 0x40 to 0x7f.
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub struct VendorCommand(u8);

impl VendorCommand {
    pub const FIRST: u8 = 0x40;
    pub const LAST: u8 = 0x7f;
}


impl TryFrom<u8> for VendorCommand {
    type Error = ();

    fn try_from(from: u8) -> core::result::Result<Self, ()> {
        match from {
            // code if code >= Self::FIRST && code <= Self::LAST => Ok(VendorCommand(code)),
            code @ Self::FIRST..=Self::LAST => Ok(VendorCommand(code)),
            // TODO: replace with Command::Unknown and infallible Try
            _ => Err(()),
        }
    }
}

impl Into<u8> for VendorCommand {
    fn into(self) -> u8 {
        self.0
    }
}

impl Into<u8> for Command {
    fn into(self) -> u8 {
        match self {
            Command::Ping => 0x01,
            Command::Msg => 0x03,
            Command::Init => 0x06,
            Command::Error => 0x3f,
            Command::Wink => 0x08,
            Command::Lock => 0x04,
            Command::Cbor => 0x10,
            Command::Cancel => 0x11,
            Command::KeepAlive => 0x3b,
            Command::Vendor(command) => command.into(),
        }
    }
}


#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(unused)]
enum State {
    Idle,

    // if request payload data is larger than one packet
    Receiving((Request, MessageState)),

    // the request message is ready, need to dispatch to "app"
    // Dispatching(Request)

    // the request message is dispatched to app, waiting for it to be processed
    Processing(Request),

    ResponsePending(Response),
    Sending((Response, MessageState)),
}

pub struct Pipe<'alloc, Authenticator, Bus>
where
    Authenticator: AuthenticatorApi,
    Bus: UsbBus,
{
    read_endpoint: EndpointOut<'alloc, Bus>,
    write_endpoint: EndpointIn<'alloc, Bus>,
    state: State,

    authenticator: &'alloc mut Authenticator,

    // shared between requests and responses, due to size
    buffer: [u8; MESSAGE_SIZE],

    // we assign channel IDs one by one, this is the one last assigned
    // TODO: move into "app"
    last_channel: u32,

}

impl<'alloc, Authenticator, Bus> Pipe<'alloc, Authenticator, Bus>
where
    Authenticator: AuthenticatorApi,
    Bus: UsbBus,
{
    pub(crate) fn new(
        read_endpoint: EndpointOut<'alloc, Bus>,
        write_endpoint: EndpointIn<'alloc, Bus>,
        authenticator: &'alloc mut Authenticator,
    ) -> Self
    {
        Self {
            read_endpoint,
            write_endpoint,
            state: State::Idle,
            authenticator,
            buffer: [0u8; MESSAGE_SIZE],
            last_channel: 0,
        }
    }

    pub fn read_address(&self) -> EndpointAddress {
        self.read_endpoint.address()
    }

    pub fn write_address(&self) -> EndpointAddress {
        self.write_endpoint.address()
    }

    // used to generate the configuration descriptors
    pub(crate) fn read_endpoint(&self) -> &EndpointOut<'alloc, Bus> {
        &self.read_endpoint
    }

    // used to generate the configuration descriptors
    pub(crate) fn write_endpoint(&self) -> &EndpointIn<'alloc, Bus> {
        &self.write_endpoint
    }

    pub(crate) fn read_and_handle_packet(&mut self) {
        // hprintln!("got a packet!").ok();
        let mut packet = [0u8; PACKET_SIZE];
        match self.read_endpoint.read(&mut packet) {
            Ok(PACKET_SIZE) => {},
            Ok(_) => {
                // error handling?
                // from spec: "Packets are always fixed size (defined by the endpoint and
                // HID report descriptors) and although all bytes may not be needed in a
                // particular packet, the full size always has to be sent.
                // Unused bytes SHOULD be set to zero."
                // hprintln!("OK but size {}", size).ok();
                return;
            },
            // usb-device lists WouldBlock or BufferOverflow as possible errors.
            // both should not occur here, and we can't do anything anyway.
            // Err(UsbError::WouldBlock) => { return; },
            // Err(UsbError::BufferOverflow) => { return; },
            Err(_) => {
                // hprintln!("error no {}", error as i32).ok();
                return;
            },
        };

        let channel = u32::from_be_bytes(packet[..4].try_into().unwrap());
        // hprintln!("channel {}", channel).ok();
        let is_initialization = (packet[4] >> 7) != 0;
        // hprintln!("is_initialization {}", is_initialization).ok();

        if is_initialization {
            // case of initialization packet

            if !(self.state == State::Idle) {
                // TODO: should we buffer "busy errors" and send them?
                // vs. just failing silently
                return;
            }

            let command_number = packet[4] & !0x80;
            // hprintln!("command number {}", command_number).ok();
            let command = match Command::try_from(command_number) {
                Ok(command) => command,
                // `solo ls` crashes here as it uses command 0x86
                Err(_) => { return; },
            };

            // can't actually fail
            let length = u16::from_be_bytes(packet[5..][..2].try_into().unwrap());

            let request = Request { channel, command, length };
            // hprintln!("request is {:?}", &request).ok();

            if length > MESSAGE_SIZE as u16 {
                // non-conforming client - we disregard it
                // TODO: error msg-too-long
                return;
            }

            // TODO: add some checks that request.length is OK.
            // e.g., CTAPHID_INIT should have payload of length 8.

            // hprintln!("receiving message of length {}", length).ok();
            if length > PACKET_SIZE as u16 - 7 {
                // store received part of payload,
                // prepare for continuation packets
                self.buffer[..PACKET_SIZE - 7]
                    .copy_from_slice(&packet[7..]);
                self.state = State::Receiving((request, {
                    let state = MessageState::default();
                    // hprintln!("got {} so far", state.transmitted).ok();
                    state
                }));
                // we're done... wait for next packet
                return;
            } else {
                // request fits in one packet
                self.buffer[..length as usize]
                    .copy_from_slice(&packet[7..][..length as usize]);
                self.state = State::Processing(request);
                self.dispatch_request();
                return;
            }
        } else {
            // case of continuation packet
            match self.state {
                State::Receiving((request, mut message_state)) => {
                    let sequence = packet[4];
                    // hprintln!("receiving continuation packet {}", sequence).ok();
                    if sequence != message_state.next_sequence {
                        // error handling?
                        // hprintln!("wrong sequence for continuation packet, expected {} received {}",
                        //           message_state.next_sequence, sequence).ok();
                        return;
                    }
                    if channel != request.channel {
                        // error handling?
                        // hprintln!("wrong channel for continuation packet, expected {} received {}",
                        //           request.channel, channel).ok();
                        return;
                    }

                    let payload_length = request.length as usize;
                    if message_state.transmitted + (PACKET_SIZE - 5) < payload_length {
                        // hprintln!("transmitted {} + (PACKET_SIZE - 5) < {}",
                        //           message_state.transmitted, payload_length).ok();
                        // store received part of payload
                        self.buffer[message_state.transmitted..][..PACKET_SIZE - 5]
                            .copy_from_slice(&packet[5..]);
                        message_state.absorb_packet();
                        self.state = State::Receiving((request, message_state));
                        // hprintln!("absorbed packet, awaiting next").ok();
                        return;
                    } else {
                        let missing = request.length as usize - message_state.transmitted;
                        self.buffer[message_state.transmitted..payload_length]
                            .copy_from_slice(&packet[5..][..missing]);
                        self.state = State::Processing(request);
                        // hprintln!("got all we need, let's dispatch").ok();
                        self.dispatch_request();
                    }
                },
                _ => {
                    // unexpected continuation packet
                    return;
                },
            }
        }
    }

    fn dispatch_request(&mut self) {
        // TODO: can we guarantee only being called in this state?
        if let State::Processing(request) = self.state {
            // dispatch request further
            match request.command {
                Command::Init => {
                    // hprintln!("command INIT!").ok();
                    // hprintln!("data: {:?}", &self.buffer[..request.length as usize]).ok();
                    match request.channel {
                        // broadcast channel ID - request for assignment
                        0xFFFF_FFFF => {
                            if request.length != 8 {
                                // error
                            } else {
                                self.last_channel += 1;
                                // hprintln!(
                                //     "assigned channel {}", self.last_channel).ok();
                                let _nonce = &self.buffer[..8];
                                let response = Response {
                                    channel: 0xFFFF_FFFF,
                                    command: request.command,
                                    length: 17,
                                };

                                self.buffer[8..12].copy_from_slice(&self.last_channel.to_be_bytes());
                                // CTAPHID protocol version
                                self.buffer[12] = 2;
                                // major device version number
                                self.buffer[13] = 0;
                                // minor device version number
                                self.buffer[14] = 0;
                                // build device version number
                                self.buffer[15] = 0;
                                // capabilities flags
                                // 0x1: implements WINK
                                // 0x4: implements CBOR
                                // 0x8: does not implement MSG
                                // self.buffer[16] = 0x01 | 0x08;
                                self.buffer[16] = 0x01 | 0x04;
                                self.start_sending(response);
                            }
                        },
                        0 => {
                            // this is an error / reserved number
                        },
                        _ => {
                            // this is assumedly the active channel,
                            // already allocated to a client
                            // TODO: "reset"
                        }
                    }
                },

                Command::Ping => {
                    // hprintln!("received PING!").ok();
                    // hprintln!("data: {:?}", &self.buffer[..request.length as usize]).ok();
                    let response = Response::from_request_and_size(request, request.length as usize);
                    self.start_sending(response);
                },

                Command::Wink => {
                    // hprintln!("received WINK!").ok();
                    // TODO: request.length should be zero
                    // TODO: callback "app"
                    let response = Response::from_request_and_size(request, 1);
                    self.start_sending(response);
                },

                Command::Cbor => {
                    // hprintln!("command CBOR!").ok();
                    self.handle_cbor(request);
                },

                Command::Msg => {
                    // hprintln!("command MSG!").ok();
                    self.handle_msg(request);
                },

                // TODO: handle other requests
                _ => {
                    // hprintln!("unknown command {:?}", request.command).ok();
                },
            }
        }
    }

    fn handle_msg(&mut self, request: Request) {
        // this is the U2F/CTAP1 layer.
        // we handle it by mapping to CTAP2, similar to how user agents
        // map CTAP2 to CTAP1.
        // hprintln!("data = {:?}", &self.buffer[..request.length as usize]).ok();

        let command = ctap1::Command::try_from(&self.buffer[..request.length as usize]);
        match command {
            Err(error) => {
                // hprintln!("ERROR").ok();
                self.buffer[..2].copy_from_slice(&(error as u16).to_be_bytes());
                let response = Response::from_request_and_size(request, 2);
                self.start_sending(response);
            },
            Ok(command) => {
                match command {
                    ctap1::Command::Version => {
                        // hprintln!("U2F_VERSION").ok();
                        // GetVersion
                        // self.buffer[0] = 0;
                        self.buffer[..6].copy_from_slice(b"U2F_V2");
                        // self.buffer[6..][..2].copy_from_slice(ctap1::NoError::to_be_bytes());
                        self.buffer[6..][..2].copy_from_slice(&(ctap1::NO_ERROR).to_be_bytes());
                        let response = Response::from_request_and_size(request, 8);
                        // hprintln!("sending response: {:x?}", &self.buffer[..response.length as usize]).ok();
                        self.start_sending(response);
                    },
                    ctap1::Command::Register(_register) => {
                        // hprintln!("command {:?}", &register).ok();
                        self.buffer[..2].copy_from_slice(&(ctap1::Error::InsNotSupported as u16).to_be_bytes());
                        let response = Response::from_request_and_size(request, 1);
                        self.start_sending(response);
                    },
                    ctap1::Command::Authenticate(_authenticate) => {
                        // hprintln!("command {:?}", &authenticate).ok();
                        self.buffer[..2].copy_from_slice(&(ctap1::Error::InsNotSupported as u16).to_be_bytes());
                        let response = Response::from_request_and_size(request, 1);
                        self.start_sending(response);
                    }
                }
            }
        }
    }

    fn handle_cbor(&mut self, request: Request) {
        let data = &self.buffer[..request.length as usize];
        // hprintln!("data: {:?}", data).ok();

        if data.len() < 1 {
            return;
        }

        let operation = match Operation::try_from(data[0]) {
            Ok(operation) => {
                // hprintln!("Operation  {:?}", &operation).ok();
                operation
            },
            Err(_) => {
                // hprintln!("Unknown operation code {:x?}", data[0]).ok();
                return;
            },
        };

        match operation {
            Operation::GetAssertion => {
                // hprintln!("received authenticatorGetAssertion").ok();
                // hprintln!("with data: {:?}", &self.buffer[1..request.length as usize]).ok();

                // let buffer_backup = self.buffer.clone();

                // let mut deserializer = serde_cbor::de::Deserializer::from_mut_slice(&mut self.buffer[1..]);
                // // let params: GetAssertionParameters =
                // //     serde::de::Deserialize::deserialize(&mut deserializer).unwrap();

                // // hprintln!("params: {:?}", &params).ok();
                // let params: GetAssertionParameters = match serde::de::Deserialize::deserialize(&mut deserializer) {
                let params: GetAssertionParameters = match crate::types::cbor_deserialize(&mut self.buffer[1..]) {
                    Ok(params) => params,
                    Err(_error) => {
                        // hprintln!("error decoding GetAssertionParameters: {:?}", error).ok();
                        // hprintln!("from data: {:?}", &buffer_backup[1..request.length as usize]).ok();
                        self.buffer[0] = AuthenticatorError::InvalidCbor as u8;
                        let response = Response::from_request_and_size(request, 1);
                        self.start_sending(response);
                        return;
                    }
                };

                match self.authenticator.get_assertions(&params) {
                    Err(error) => {
                        // hprintln!("error getting assertions: {:?}", &error).ok();
                        self.buffer[0] = error as u8;
                        let response = Response::from_request_and_size(request, 1);
                        self.start_sending(response);
                    },
                    Ok(assertion_responses) => {
                        // hprintln!("got assertion_responses: {:?}", &assertion_responses).ok();
                        self.buffer[0] = 0;

                        let writer = serde_cbor::ser::SliceWrite::new(&mut self.buffer[1..]);
                        let mut ser = serde_cbor::Serializer::new(writer)
                            // .packed_format()
                            // .pack_starting_with(1)
                            // .pack_to_depth(2)
                        ;

                        let attestation_object = &assertion_responses[0];
                        attestation_object.serialize(&mut ser).unwrap();

                        let writer = ser.into_inner();
                        let size = 1 + writer.bytes_written();

                        // hprintln!("sending response: {:?}", attestation_object).ok();
                        // hprintln!("serialized response: {:?}", &self.buffer[1..size]).ok();

                        let response = Response::from_request_and_size(request, size);
                        self.start_sending(response);
                    },
                }
            },

            Operation::MakeCredential => {
                // hprintln!("received authenticatorMakeCredential").ok();
                // let buffer_backup = self.buffer.clone();

                // let mut deserializer = serde_cbor::de::Deserializer::from_mut_slice(&mut self.buffer[1..]);
                //     // .packed_starts_with(1);
                // let params: MakeCredentialParameters = match serde::de::Deserialize::deserialize(&mut deserializer) {
                let params: MakeCredentialParameters = match crate::types::cbor_deserialize(&mut self.buffer[1..]) {
                    Ok(params) => params,
                    Err(_error) => {
                        // hprintln!("error decoding MakeCredentialParameters: {:?}", error).ok();
                        // hprintln!("from data: {:?}", &buffer_backup[1..request.length as usize]).ok();
                        self.buffer[0] = AuthenticatorError::InvalidCbor as u8;
                        let response = Response::from_request_and_size(request, 1);
                        self.start_sending(response);
                        return;
                    }
                };

                // hprintln!("params: {:?}", &params).ok();

                match self.authenticator.make_credential(&params) {
                    Err(error) => {
                        // hprintln!("error making credentials: {:?}", &error).ok();
                        self.buffer[0] = error as u8;
                        let response = Response::from_request_and_size(request, 1);
                        self.start_sending(response);
                    },
                    Ok(attestation_object) => {
                        // hprintln!("generated attestation object: {:?}", &attestation_object).ok();
                        self.buffer[0] = 0;

                        let writer = serde_cbor::ser::SliceWrite::new(&mut self.buffer[1..]);
                        let mut ser = serde_cbor::Serializer::new(writer)
                            // .packed_format()
                            // .pack_starting_with(1)
                            // .pack_to_depth(1)
                        ;

                        attestation_object.serialize(&mut ser).unwrap();

                        let writer = ser.into_inner();
                        let size = 1 + writer.bytes_written();

                        // hprintln!("sending response: {:?}", &attestation_object).ok();
                        // hprintln!("serialized response: {:?}", &self.buffer[1..size]).ok();

                        let response = Response::from_request_and_size(request, size);
                        self.start_sending(response);
                    },
                }
            },

            Operation::GetInfo => {
                // hprintln!("received authenticatorGetInfo").ok();

                let authenticator_info = self.authenticator.get_info();
                // hprintln!("authenticator_info = {:?}", &authenticator_info).ok();

                // status: 0  = success;
                self.buffer[0] = 0;
                // actual payload
                let writer = serde_cbor::ser::SliceWrite::new(&mut self.buffer[1..]);
                let mut ser = serde_cbor::Serializer::new(writer)
                    // .packed_format()
                    // .pack_starting_with(1)
                    // .pack_to_depth(1)
                ;

                // hprintln!("returning info {:?}", &authenticator_info).ok();
                authenticator_info.serialize(&mut ser).unwrap();

                let writer = ser.into_inner();
                let size = 1 + writer.bytes_written();

                // let mut scratch = [0u8; 128];
                // let mut a: AuthenticatorInfo = serde_cbor::de::from_slice_with_scratch(
                //     &self.buffer[1..size], &mut scratch).unwrap()/
                // let mut a: AuthenticatorInfo = serde_cbor::de::from_mut_slice(
                //     &mut self.buffer[1..size]).unwrap()/

                // let mut scratch = [0u8; 128];
                // let authn: AuthenticatorInfo = serde_cbor::de::from_slice_with_scratch(
                //     &self.buffer[1..size], &mut scratch).unwrap();

                // hprintln!("using serde, wrote {} bytes: {:x?}",
                //           size, &self.buffer[..size]).ok();
                let response = Response::from_request_and_size(request, size);
                self.start_sending(response);
            },

            Operation::Reset => {
                // hprintln!("received authenticatorReset").ok();
                match self.authenticator.reset() {
                    Ok(_) =>  { self.buffer[0] = 0; },
                    Err(error) => { self.buffer[0] = error as u8; },
                }
                let response = Response::from_request_and_size(request, 1);
                self.start_sending(response);
            },

            _ => {
                // hprintln!("Operation {:?} not implemented", operation).ok();
                return;
            },
        }
    }

    fn start_sending(&mut self, response: Response) {
        self.state = State::ResponsePending(response);
        self.maybe_write_packet();
    }

    // called from poll, and when a packet has been sent
    pub(crate) fn maybe_write_packet(&mut self) {
        match self.state {
            State::ResponsePending(response) => {

                // zeros leftover bytes
                let mut packet = [0u8; PACKET_SIZE];
                packet[..4].copy_from_slice(&response.channel.to_be_bytes());
                // packet[4] = response.command.into() | 0x80u8;
                packet[4] = response.command.into_u8() | 0x80;
                packet[5..7].copy_from_slice(&response.length.to_be_bytes());

                let fits_in_one_packet = 7 + response.length as usize <= PACKET_SIZE;
                if fits_in_one_packet {
                    packet[7..][..response.length as usize]
                        .copy_from_slice( &self.buffer[..response.length as usize]);
                    self.state = State::Idle;
                } else {
                    packet[7..].copy_from_slice(&self.buffer[..PACKET_SIZE - 7]);
                }

                // try actually sending
                // hprintln!("attempting to write init packet {:?}, {:?}",
                //           &packet[..32], &packet[32..]).ok();
                let result = self.write_endpoint.write(&packet);

                match result {
                    Err(UsbError::WouldBlock) => {
                        // fine, can't write try later
                        // this shouldn't happen probably
                    },
                    Err(_) => {
                        // hprintln!("weird USB errrorrr").ok();
                        panic!("unexpected error writing packet!");
                    },
                    Ok(PACKET_SIZE) => {
                        // goodie, this worked
                        if fits_in_one_packet {
                            self.state = State::Idle;
                            // hprintln!("StartSent {} bytes, idle again", response.length).ok();
                            // hprintln!("IDLE again").ok();
                        } else {
                            self.state = State::Sending((response, MessageState::default()));
                            // hprintln!(
                            //     "StartSent {} of {} bytes, waiting to send again",
                            //     PACKET_SIZE - 7, response.length).ok();
                            // hprintln!("State: {:?}", &self.state).ok();
                        }
                    },
                    Ok(_) => {
                        // hprintln!("short write").ok();
                        panic!("unexpected size writing packet!");
                    },
                };
            },

            State::Sending((response, mut message_state)) => {
                // hprintln!("in StillSending").ok();
                let mut packet = [0u8; PACKET_SIZE];
                packet[..4].copy_from_slice(&response.channel.to_be_bytes());
                packet[4] = message_state.next_sequence;

                let sent = message_state.transmitted;
                let remaining = response.length as usize - sent;
                let last_packet = 5 + remaining <= PACKET_SIZE;
                if last_packet {
                    packet[5..][..remaining].copy_from_slice(
                        &self.buffer[message_state.transmitted..][..remaining]);
                } else {
                    packet[5..].copy_from_slice(
                        &self.buffer[message_state.transmitted..][..PACKET_SIZE - 5]);
                }

                // try actually sending
                // hprintln!("attempting to write cont packet {:?}, {:?}",
                //           &packet[..32], &packet[32..]).ok();
                let result = self.write_endpoint.write(&packet);

                match result {
                    Err(UsbError::WouldBlock) => {
                        // fine, can't write try later
                        // this shouldn't happen probably
                        // hprintln!("can't send seq {}, write endpoint busy",
                        //           message_state.next_sequence).ok();
                    },
                    Err(_) => {
                        // hprintln!("weird USB error").ok();
                        panic!("unexpected error writing packet!");
                    },
                    Ok(PACKET_SIZE) => {
                        // goodie, this worked
                        if last_packet {
                            self.state = State::Idle;
                            // hprintln!("in IDLE state after {:?}", &message_state).ok();
                        } else {
                            message_state.absorb_packet();
                            // DANGER! destructuring in the match arm copies out
                            // message state, so need to update state
                            // hprintln!("sent one more, now {:?}", &message_state).ok();
                            self.state = State::Sending((response, message_state));
                        }
                    },
                    Ok(_) => {
                        // hprintln!("short write").ok();
                        panic!("unexpected size writing packet!");
                    },
                };
            },

            // nothing to send
            _ => {
            },
        }
    }
}
