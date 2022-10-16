
# GENERATED: This file is automatically updated by 'Bump dependencies', local changes will be overwritten!

FROM mverleg/rust_nightly_musl_base:deps_2022-10-01_35

# Copy the code (all except .dockerignore).
COPY ./ ./

# Build (for test)
ARG TEST=1
RUN find . -name target -prune -o -type f &&\
    touch -c build.rs src/main.rs src/lib.rs &&\
    if [ "$TEST" != 0 ] ; then \
        cargo build --all-features --tests --locked; \
    else \
        cargo build --all-features --locked; \
    fi

# Test
RUN if [ "$TEST" != 0 ] ; then  \
        cargo --offline test --all-features;  \
    else  \
        echo SKIPPED;  \
    fi

# Lint
ARG LINT=1
ARG STRICT=1
RUN if [ "$LINT" != 0 ] ; then  \
        if [ "$STRICT" != 0 ] ; then  \
            cargo --offline clippy --all-features --tests -- -D warnings;  \
        else  \
            cargo --offline clippy --all-features --tests;  \
        fi \
    else  \
        echo SKIPPED;  \
    fi

# Style
ARG FMT=1
RUN if [ "$FMT" != 0 ] ; then  \
        cargo --offline fmt --all -- --check;  \
    else  \
        echo SKIPPED;  \
    fi

