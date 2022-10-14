
# GENERATED: This file is automatically updated by 'Bump dependencies', local changes will be overwritten!

FROM mverleg/rust_nightly_musl_base:deps_2022-10-01_35 AS build

ARG BIN
ARG TARGET=x86_64-unknown-linux-musl

RUN test -n "$BIN" || { echo "build arg BIN is required: cargo binary" 1>&2 && exit 1; } &&\
    test -n "$TARGET" || { echo "build arg TARGET is required: build target" 1>&2 && exit 2; }

# Add target
RUN rustup target add "$TARGET"

# Copy the code (all except .dockerignore).
COPY ./ ./

# Build (for release)
RUN find . -name target -prune -o -type f &&\
    touch -c build.rs src/main.rs src/lib.rs &&\
    cargo build --bin "$BIN" --target "$TARGET" --release

# Copy executable
RUN find . -wholename "*/target/*" -name "$BIN" -type f -executable -print -exec cp {} /"$BIN" \; &&\
    test -f /"$BIN"


# Second stage image to decrease size
FROM scratch AS executable

ARG BIN

ENV PATH=/
ENV RUST_BACKTRACE=1
ENV RUST_LOG='warn'
WORKDIR /code

COPY --from=build /"$BIN" /"$BIN"

ENTRYPOINT [ "$BIN" ]

