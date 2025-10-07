FROM debian:12-slim

ARG TARGETARCH
ARG TARGETPLATFORM

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY picolayer /usr/local/bin/picolayer

RUN chmod +x /usr/local/bin/picolayer

ENTRYPOINT ["/usr/local/bin/picolayer"]
CMD ["--help"]
