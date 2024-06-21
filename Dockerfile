FROM rust:1.77.1
RUN apt-get update && \
    apt-get install -y python3 python3-toml git curl llvm clang libclang-dev gcc-arm-none-eabi libc6-dev-i386 make wget zip
RUN cargo install flip-link cargo-binutils
RUN rustup target add thumbv7em-none-eabihf thumbv8m.main-none-eabi
RUN rustup component add llvm-tools-preview clippy rustfmt
RUN cargo install --git https://github.com/Nitrokey/nitrokey-ci --rev 934fe3d441cfd15901a88aa5aff712edd29a78aa --locked
RUN cargo install --git https://github.com/Nitrokey/repometrics --rev 98ffa20ddded8f09c0ef252b4e93ec6a9792f9dc --locked
RUN rustup install nightly-2024-04-01
RUN rustup component add rust-src --toolchain nightly-2024-04-01
WORKDIR /app
