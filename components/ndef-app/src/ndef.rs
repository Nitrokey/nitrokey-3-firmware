use core::sync::atomic::{AtomicUsize, Ordering};

use apdu_app::{CommandView, Data, Interface};
use iso7816::{Instruction, Status};

static URL_RX_FN: AtomicUsize = AtomicUsize::new(0);
static URL_TX_FN: AtomicUsize = AtomicUsize::new(0);
static URL_RX_COUNT_FN: AtomicUsize = AtomicUsize::new(0);
static URL_TX_COUNT_FN: AtomicUsize = AtomicUsize::new(0);
static URL_IRQ_COUNT_FN: AtomicUsize = AtomicUsize::new(0);
static URL_IRQ_FIRST_FN: AtomicUsize = AtomicUsize::new(0);

pub fn install_url_rx_reader(f: fn() -> u32) {
    URL_RX_FN.store(f as usize, Ordering::Release);
}

pub fn install_url_tx_reader(f: fn() -> u32) {
    URL_TX_FN.store(f as usize, Ordering::Release);
}

pub fn install_url_rx_count_reader(f: fn() -> u32) {
    URL_RX_COUNT_FN.store(f as usize, Ordering::Release);
}

pub fn install_url_tx_count_reader(f: fn() -> u32) {
    URL_TX_COUNT_FN.store(f as usize, Ordering::Release);
}

pub fn install_url_irq_count_reader(f: fn() -> u32) {
    URL_IRQ_COUNT_FN.store(f as usize, Ordering::Release);
}

pub fn install_url_irq_first_reader(f: fn() -> u32) {
    URL_IRQ_FIRST_FN.store(f as usize, Ordering::Release);
}

fn read_via(slot: &AtomicUsize) -> Option<u32> {
    let raw = slot.load(Ordering::Acquire);
    if raw == 0 {
        return None;
    }
    // SAFETY: only the matching `install_*` writes this slot.
    let f: fn() -> u32 = unsafe { core::mem::transmute(raw) };
    let v = f();
    if v == 0 {
        None
    } else {
        Some(v)
    }
}

#[derive(Clone, Copy)]
enum ReaderKind {
    Cc,
    Ndef,
}

const URL_BASE: &[u8] = b"nitrokey.com/";

// Bound for the dynamic NDEF buffer:
//   2  (file length)
// + 4  (record header: TNF, type-len, payload-len, type)
// + 1  (URI prefix code 0x02 = "https://www.")
// + URL_BASE.len() (13)
// + QUERY_PREFIX.len() (3)
// + 10 (max u32 decimal digits)
// = 33; round up.
const NDEF_BUF_LEN: usize = 96;

pub struct App {
    reader_kind: ReaderKind,
    ndef_buf: [u8; NDEF_BUF_LEN],
    ndef_len: usize,
}

impl App {
    pub const CAPABILITY_CONTAINER: [u8; 15] = [
        0x00, 0x0f, /* CCEN_HI, CCEN_LOW */
        0x20, /* VERSION */
        0x00, 0x7f, /* MLe_HI, MLe_LOW */
        0x00, 0x7f, /* MLc_HI, MLc_LOW */
        /* TLV */
        0x04, 0x06, 0xe1, 0x04, 0x00, 0x7f, 0x00, 0x00,
    ];

    pub fn new() -> App {
        let mut app = App {
            reader_kind: ReaderKind::Ndef,
            ndef_buf: [0; NDEF_BUF_LEN],
            ndef_len: 0,
        };
        app.rebuild_ndef();
        app
    }

    fn current_reader(&self) -> &[u8] {
        match self.reader_kind {
            ReaderKind::Cc => &Self::CAPABILITY_CONTAINER,
            ReaderKind::Ndef => &self.ndef_buf[..self.ndef_len],
        }
    }

    /// Render the NDEF URI record into `self.ndef_buf`, picking up the
    /// current measured-µs values if readers are installed.
    fn rebuild_ndef(&mut self) {
        // Build the URL bytes after the prefix code (0x02 = "https://www.").
        let mut url = [0u8; NDEF_BUF_LEN];
        let mut url_len = 0;
        url[..URL_BASE.len()].copy_from_slice(URL_BASE);
        url_len += URL_BASE.len();

        // Compose query string of the form
        // "?r=<rx>&n=<rxc>&i=<irqc>&t=<tx>&s=<txc>", skipping any value
        // whose reader is missing or returns 0.
        let rx = read_via(&URL_RX_FN);
        let rxc = read_via(&URL_RX_COUNT_FN);
        let irqc = read_via(&URL_IRQ_COUNT_FN);
        let irqf = read_via(&URL_IRQ_FIRST_FN);
        let tx = read_via(&URL_TX_FN);
        let txc = read_via(&URL_TX_COUNT_FN);
        let mut first = true;
        let mut append_kv = |key: &[u8], v: u32, url: &mut [u8], url_len: &mut usize| {
            url[*url_len] = if first { b'?' } else { b'&' };
            *url_len += 1;
            first = false;
            url[*url_len..*url_len + key.len()].copy_from_slice(key);
            *url_len += key.len();
            url[*url_len] = b'=';
            *url_len += 1;
            *url_len += format_u32(v, &mut url[*url_len..]);
        };
        if let Some(v) = rx {
            append_kv(b"r", v, &mut url, &mut url_len);
        }
        if let Some(v) = rxc {
            append_kv(b"n", v, &mut url, &mut url_len);
        }
        if let Some(v) = irqc {
            append_kv(b"i", v, &mut url, &mut url_len);
        }
        if let Some(v) = irqf {
            append_kv(b"f", v, &mut url, &mut url_len);
        }
        if let Some(v) = tx {
            append_kv(b"t", v, &mut url, &mut url_len);
        }
        if let Some(v) = txc {
            append_kv(b"s", v, &mut url, &mut url_len);
        }

        let payload_len = 1 + url_len; // prefix code byte + url chars
        let record_len = 4 + payload_len; // TNF + type-len + payload-len + type + payload

        debug_assert!(2 + record_len <= NDEF_BUF_LEN);
        debug_assert!(record_len <= u8::MAX as usize);
        debug_assert!(payload_len <= u8::MAX as usize);

        self.ndef_buf[0] = 0x00;
        self.ndef_buf[1] = record_len as u8;
        self.ndef_buf[2] = 0xd1; // TNF: well-known + MB + ME + SR
        self.ndef_buf[3] = 0x01; // type length
        self.ndef_buf[4] = payload_len as u8;
        self.ndef_buf[5] = 0x55; // type "U"
        self.ndef_buf[6] = 0x02; // URI prefix: "https://www."
        self.ndef_buf[7..7 + url_len].copy_from_slice(&url[..url_len]);
        self.ndef_len = 7 + url_len;
    }
}

fn format_u32(mut n: u32, buf: &mut [u8]) -> usize {
    if n == 0 {
        buf[0] = b'0';
        return 1;
    }
    // Count digits.
    let mut digits = 0;
    let mut tmp = n;
    while tmp > 0 {
        digits += 1;
        tmp /= 10;
    }
    let mut i = digits;
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    digits
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl iso7816::App for App {
    fn aid(&self) -> iso7816::Aid {
        iso7816::Aid::new(&[0xD2u8, 0x76, 0x00, 0x00, 0x85, 0x01, 0x01])
    }
}

impl<const R: usize> apdu_app::App<R> for App {
    fn select(
        &mut self,
        _interface: Interface,
        _apdu: CommandView<'_>,
        _reply: &mut Data<R>,
    ) -> apdu_app::Result {
        debug_now!("Got Select");
        Ok(())
    }

    fn deselect(&mut self) {}

    fn call(
        &mut self,
        _type: Interface,
        apdu: CommandView<'_>,
        reply: &mut Data<R>,
    ) -> apdu_app::Result {
        debug_now!("Got call: {apdu:02x?}");
        let instruction = apdu.instruction();
        let p1 = apdu.p1;
        let p2 = apdu.p2;
        let expected = apdu.expected();
        let payload = apdu.data();

        match instruction {
            Instruction::Select => {
                if payload.starts_with(&[0xE1u8, 0x03]) {
                    self.reader_kind = ReaderKind::Cc;
                    Ok(())
                } else if payload.starts_with(&[0xE1u8, 0x04]) {
                    // Refresh the URL with the current measured-µs snapshot
                    // before the reader starts pulling bytes out of the file.
                    self.rebuild_ndef();
                    self.reader_kind = ReaderKind::Ndef;
                    Ok(())
                } else {
                    Err(Status::NotFound)
                }
            }
            Instruction::ReadBinary => {
                let reader = self.current_reader();
                let offset = (((p1 & 0xef) as usize) << 8) | p2 as usize;
                let len_to_read = if expected > (reader.len() - offset) {
                    reader.len() - offset
                } else if expected > 0 {
                    expected
                } else {
                    reader.len() - offset
                };

                reply
                    .extend_from_slice(&reader[offset..offset + len_to_read])
                    .ok();
                Ok(())
            }
            _ => Err(Status::ConditionsOfUseNotSatisfied),
        }
    }
}
