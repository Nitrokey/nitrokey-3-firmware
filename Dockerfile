FROM rust:1.65
RUN apt-get update && \
    apt-get install -y python3 python3-toml git curl llvm clang libclang-dev gcc-arm-none-eabi libc6-dev-i386 make wget zip
RUN cargo install flip-link cargo-binutils
RUN rustup target add thumbv7em-none-eabihf thumbv8m.main-none-eabi
RUN rustup +nightly-2022-11-13 target add thumbv7em-none-eabihf thumbv8m.main-none-eabi
RUN rustup component add llvm-tools-preview
WORKDIR /app
