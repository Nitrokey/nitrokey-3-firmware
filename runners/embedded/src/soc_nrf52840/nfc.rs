use nfc_device::traits::nfc;

pub struct NrfNfc {
    nfc_pac: nrf52840_pac::NFCT,
}

impl NrfNfc {
    pub fn new(pac: nrf52840_pac::NFCT) -> Self {
        Self { nfc_pac: pac }
    }
}

impl nfc::Device for NrfNfc {
    fn read(&mut self, buf: &mut [u8]) -> Result<nfc::State, nfc::Error> {
        Err(nfc::Error::NoActivity)
    }

    fn send(&mut self, buf: &[u8]) -> Result<(), nfc::Error> {
        Ok(())
    }

    fn frame_size(&self) -> usize {
        0
    }
}
