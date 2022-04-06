FROM rust:1.59.0-bullseye

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
RUN git clone https://github.com/ChainSafe/api3-rust
RUN cd api3-rust && cargo test --all --all-features
RUN cargo --version && rustc --version
