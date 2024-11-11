FROM rust:1.80.1

RUN export DEBIAN_FRONTEND=noninteractive \
    && apt-get -qq update \
    && apt-get -qq install --no-install-recommends \
    pre-commit \
    python3-pip \
    libclang-dev \
    shellcheck \
    clang \
    curl \
    lcov \
    && rm -rf /var/lib/apt/lists/*

ENV CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    RUST_VERSION=1.80.1 \
    PATH=/usr/local/cargo/bin:$PATH

RUN rustup toolchain add nightly \
    --target x86_64-unknown-linux-gnu \
    --component rustfmt miri llvm-tools-preview
RUN rustup toolchain add 1.80.1 \
    --target x86_64-unknown-linux-gnu \
    --component clippy rustfmt llvm-tools-preview
RUN rustup target add --toolchain 1.80.1 \
    x86_64-unknown-linux-musl
RUN rustup default 1.80.1-x86_64-unknown-linux-gnu
RUN cargo install --quiet --no-default-features --features native-certs cargo-deny

RUN curl --location --silent --fail --output /usr/local/bin/shfmt https://github.com/mvdan/sh/releases/download/v3.8.0/shfmt_v3.8.0_linux_amd64 && \
    chmod +x /usr/local/bin/shfmt

COPY --from=ghcr.io/staex-io/cijail:latest / /usr/local

LABEL org.opencontainers.image.source=https://github.com/igankevich/any-decoder
LABEL org.opencontainers.image.description="CI image"

CMD ["/bin/bash"]
