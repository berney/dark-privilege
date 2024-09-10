FROM rust:alpine AS build
WORKDIR /src
# Install build dependencies, toolchains, targets etc
RUN <<-EOF
    set -eux
    # Install MinGW-w64 and other necessary dependencies
    apk add --no-cache \
      mingw-w64-binutils \
      mingw-w64-crt \
      mingw-w64-gcc \
      mingw-w64-headers \
      mingw-w64-winpthreads \
      cmake \
      musl-dev \
      gcc \
      libgcc
    rustup toolchain list
    rustup target list --installed
    rustup toolchain install --force-non-host stable-x86_64-pc-windows-gnu
    #rustup toolchain install --force-non-host stable-x86_64-pc-windows-msvc
    rustup target add x86_64-pc-windows-gnu
    #rustup target add x86_64-pc-windows-msvc
EOF
COPY . .
# Build it
RUN <<-EOF
    set -eux
    cargo build
    cargo build --target x86_64-pc-windows-gnu
    #cargo build --target x86_64-pc-windows-msvc
    cargo build --release
    cargo build --release --target x86_64-pc-windows-gnu
    #cargo build --release --target x86_64-pc-windows-msvc
    cargo install --path .
EOF

FROM scratch
# XXX Should I use `--link`?
COPY --from=build /usr/local/cargo/bin/dark-privilege /
COPY --from=build /src/target/x86_64-pc-windows-gnu/release/dark-privilege.exe /x86_64-pc-windows-gnu/release/
CMD ["./dark-privilege"]
