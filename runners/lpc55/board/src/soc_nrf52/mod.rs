// pub mod traits;

// board support package
#[cfg(not(any(feature = "board-nrfdk", feature = "board-ptbproto1", feature = "board-nk3amnrf")))]
compile_error!("Please select one of the NRF52 board features.");

// #[cfg(feature = "board-ptbproto1")]
// pub mod ptbproto1;
// #[cfg(feature = "board-ptbproto1")]
// pub use ptbproto1 as specifics;
