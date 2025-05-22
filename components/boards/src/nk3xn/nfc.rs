use super::spi::Spi;
use apps::InitStatus;
use lpc55_hal::{
    drivers::{
        pins::{self, Pin},
        Timer,
    },
    peripherals::flexcomm,
    typestates::pin,
    Enabled,
};

use fm11nc08::{Configuration, Register, FM11NC08};

pub type NfcCsPin = pins::Pio1_20;
pub type NfcIrqPin = pins::Pio0_19;

pub type OldNfcChip = FM11NC08<
    Spi,
    Pin<NfcCsPin, pin::state::Gpio<pin::gpio::direction::Output>>,
    Pin<NfcIrqPin, pin::state::Gpio<pin::gpio::direction::Input>>,
>;

pub type Nt3h2111I2C = lpc55_hal::I2cMaster<
    pins::Pio1_20,
    pins::Pio1_21,
    flexcomm::I2c4,
    (
        Pin<pins::Pio1_20, pin::state::Special<pin::function::FC4_TXD_SCL_MISO_WS>>,
        Pin<pins::Pio1_21, pin::state::Special<pin::function::FC4_RXD_SDA_MOSI_DATA>>,
    ),
>;

pub type Nt3h2111FdPin = Pin<pins::Pio1_9, pin::state::Gpio<pins::direction::Input>>;

pub type Nt3h2111 = nt3h2111::Nt3h2111<Nt3h2111I2C, Nt3h2111FdPin>;
pub type NfcChip = Nt3h2111;

pub fn try_setup(
    spi: Spi,
    gpio: &mut lpc55_hal::Gpio<Enabled>,
    iocon: &mut lpc55_hal::Iocon<Enabled>,
    nfc_irq: Pin<NfcIrqPin, pin::state::Gpio<pin::gpio::direction::Input>>,
    // fm: &mut NfcChip,
    timer: &mut Timer<impl lpc55_hal::peripherals::ctimer::Ctimer<Enabled>>,
    status: &mut InitStatus,
) -> Option<OldNfcChip> {
    // Start unselected.
    let nfc_cs = NfcCsPin::take()
        .unwrap()
        .into_gpio_pin(iocon, gpio)
        .into_output_high();

    let mut fm = FM11NC08::new(spi, nfc_cs, nfc_irq).enabled();

    #[allow(clippy::eq_op, clippy::identity_op)]
    // FIXME: use bitlfags to document what is being configured
    //                      no limit      2mA resistor    3.3V
    const REGU_CONFIG: u8 = (0b11 << 4) | (0b10 << 2) | (0b11 << 0);
    let current_regu_config = fm.read_reg(Register::ReguCfg);
    let current_nfc_config = fm.read_reg(Register::NfcCfg);

    // regu_config gets configured by upstream vendor testing, so we need
    // to additionally test on another value to see if eeprom is configured by us.
    let is_select_int_masked = (current_nfc_config & 1) == 1;

    if current_regu_config == 0xff {
        // No nfc chip connected
        status.insert(InitStatus::NFC_ERROR);
        info!("No NFC chip connected");
        return None;
    }

    let reconfig = (current_regu_config != REGU_CONFIG) || (is_select_int_masked);

    if reconfig {
        // info_now!("{:?}", fm.dump_eeprom() );
        // info_now!("{:?}", fm.dump_registers() );

        info!("writing EEPROM");

        let r = fm.configure(
            Configuration {
                regu: REGU_CONFIG,
                ataq: 0x4400,
                sak1: 0x04,
                sak2: 0x20,
                tl: 0x05,
                // (x[7:4], FSDI[3:0]) . FSDI[2] == 32 byte frame, FSDI[8] == 256 byte frame, 7==128byte
                t0: 0x78,
                // Support different data rates for both directions
                // Support divisor 2 / 212kbps for tx and rx
                ta: 0b10010001,
                // (FWI[b4], SFGI[b4]), (256 * 16 / fc) * 2 ^ value
                tb: 0x78,
                tc: 0x00,
                #[allow(clippy::eq_op, clippy::identity_op)]
                // enable P-on IRQ    14443-4 mode
                // FIXME: use bitlfags to document what is being configured
                nfc: (0b0 << 1) | (0b00 << 2),
            },
            timer,
        );
        if r.is_err() {
            status.insert(InitStatus::NFC_ERROR);
            info!("Eeprom failed.  No NFC chip connected?");
            return None;
        }
    } else {
        info!("EEPROM already initialized.");
    }

    // disable all interrupts except RxStart
    fm.write_reg(Register::AuxIrqMask, 0x00);
    fm.write_reg(
        Register::FifoIrqMask,
        // 0x0
        0xff
        ^ (1 << 3) /* water-level */
        ^ (1 << 1), /* fifo-full */
    );
    fm.write_reg(
        Register::MainIrqMask,
        // 0x0
        0xff ^ fm11nc08::device::Interrupt::RxStart as u8
            ^ fm11nc08::device::Interrupt::RxDone as u8
            ^ fm11nc08::device::Interrupt::TxDone as u8
            ^ fm11nc08::device::Interrupt::Fifo as u8
            ^ fm11nc08::device::Interrupt::Active as u8,
    );

    //                    no limit    rrfcfg .      3.3V
    // let regu_powered = (0b11 << 4) | (0b10 << 2) | (0b11 << 0);
    // fm.write_reg(Register::ReguCfg, regu_powered);

    Some(fm)
}
