
# GENERATED: This file is automatically updated by 'Bump dependencies', local changes will be overwritten!

FROM mverleg/rust_nightly_musl_base:deps_2022-01-01_5

# Copy the code (all except .dockerignore).
COPY ./ ./

# Build (for test)
RUN find . -name target -prune -o -type f &&\
    touch -c build.rs src/main.rs src/lib.rs &&\
    cargo build --all-features --tests --locked

# Test
RUN cargo --offline test --all-features

# Lint
RUN cargo --offline clippy --all-features --tests -- -D warnings

# Style
RUN cargo --offline fmt --all -- --check

