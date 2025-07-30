FROM rust:1.88.0
RUN apt-get update && \
    apt-get install -y python3 python3-toml git curl llvm clang libclang-dev gcc-arm-none-eabi libc6-dev-i386 make wget zip
RUN cargo install flip-link cargo-binutils
RUN rustup target add thumbv7em-none-eabihf thumbv8m.main-none-eabi
RUN rustup component add llvm-tools-preview clippy rustfmt
RUN cargo install --git https://github.com/Nitrokey/nitrokey-ci --rev 1dcacd7a89621b29403bace6cd0abb254844bf0c --locked
RUN cargo install --git https://github.com/Nitrokey/repometrics --rev 1f0f43a119e4b0412f8eae416cff96b68d62b8bd --locked
RUN rustup install nightly-2025-05-09
RUN rustup component add rust-src --toolchain nightly-2025-05-09
WORKDIR /app
