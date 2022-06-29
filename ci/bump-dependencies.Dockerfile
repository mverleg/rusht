
FROM mverleg/rust_nightly_musl_base:nodeps_2022-04-01_29

COPY ./ ./

RUN cargo upgrade && cargo update

RUN cargo --offline outdated --exit-code 1
