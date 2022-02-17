pub mod types;

#[cfg(feature = "board-nrfdk")]
pub mod board_nrfdk;
#[cfg(feature = "board-nrfdk")]
pub use board_nrfdk as board;

#[cfg(not(any(feature = "board-nrfdk")))]
compile_error!("No NRF52840 board chosen!");
