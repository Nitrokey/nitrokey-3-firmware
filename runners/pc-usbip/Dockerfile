FROM rust:latest
RUN apt update -qy && apt install -qy libclang-dev
RUN rustc --version
ENV CARGO_HOME /root/.cargo/
