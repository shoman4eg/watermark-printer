# Debian 12 builds against glibc 2.36, so the image runs on older hosts too.
FROM rust:1-bookworm AS build

# cmake is needed by aws-lc-sys (rustls).
RUN apt-get update \
    && apt-get install -y --no-install-recommends cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .
RUN cargo build --release --locked --bin watermark-printer

FROM debian:bookworm-slim

# ca-certificates for ipps://, cups-client for the -I lp destination.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates cups-client \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /src/target/release/watermark-printer /usr/local/bin/watermark-printer

EXPOSE 631
ENTRYPOINT ["watermark-printer"]
CMD ["-p", "631"]