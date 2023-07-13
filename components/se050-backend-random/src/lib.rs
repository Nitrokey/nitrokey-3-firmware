#![no_std]

use embedded_hal::blocking::delay::DelayUs;
use se050::{
    se050::{commands::GetRandom, Se050},
    t1::I2CForT1,
};
use trussed::{
    api::{reply, request, Request},
    backend::Backend,
    config::MAX_MESSAGE_LENGTH,
    types::Message,
};

delog::generate_macros!();

/// Need overhead for TLV + SW bytes
const BUFFER_LEN: usize = trussed::config::MAX_MESSAGE_LENGTH + 10;

pub struct BackendRandom<Twi, D> {
    se: Se050<Twi, D>,
    enabled: bool,
    failed_enable: Option<se050::se050::Error>,
}

impl<Twi: I2CForT1, D: DelayUs<u32>> BackendRandom<Twi, D> {
    pub fn new(se: Se050<Twi, D>) -> Self {
        BackendRandom {
            se,
            enabled: false,
            failed_enable: None,
        }
    }
}

impl<Twi: I2CForT1, D: DelayUs<u32>> Backend for BackendRandom<Twi, D> {
    type Context = ();
    fn request<P: trussed::Platform>(
        &mut self,
        _core_ctx: &mut trussed::types::CoreContext,
        _backend_ctx: &mut Self::Context,
        request: &trussed::api::Request,
        _resources: &mut trussed::service::ServiceResources<P>,
    ) -> Result<trussed::Reply, trussed::Error> {
        let Request::RandomBytes(request::RandomBytes{count}) = request else {
            return Err(trussed::Error::RequestNotAvailable);
        };
        let count = *count;

        if count >= MAX_MESSAGE_LENGTH {
            return Err(trussed::Error::MechanismParamInvalid);
        }

        if !self.enabled {
            if let Err(e) = self.se.enable() {
                self.failed_enable = Some(e);
            } else {
                self.failed_enable = None;
                self.enabled = true;
            }
        }
        if let Some(_e) = self.failed_enable {
            error!("Enabling failed: {:?}", _e);
            return Err(trussed::Error::FunctionFailed);
        }

        let mut buf = [0; BUFFER_LEN];
        let res = self
            .se
            .run_command(
                &GetRandom {
                    length: (count as u16).into(),
                },
                &mut buf,
            )
            .map_err(|_err| {
                error!("Failed to get random: {:?}", _err);
                trussed::Error::FunctionFailed
            })?;
        if res.data.len() != count {
            error!("Bad random length");
            return Err(trussed::Error::FunctionFailed);
        }
        Ok(reply::RandomBytes {
            bytes: Message::from_slice(res.data).unwrap(),
        }
        .into())
    }
}
