####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=valence
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /valence

COPY ./ .
COPY ./config.toml ./

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM cgr.dev/chainguard/glibc-dynamic:latest

USER nonroot

WORKDIR /valence

# Copy our build
COPY --from=builder /valence/target/x86_64-unknown-linux-musl/release/valence ./
COPY --from=builder /valence/config.toml ./

CMD ["/valence/valence"]
