FROM debian:12-slim

RUN apt-get update && apt-get install -y jq curl ca-certificates libc6 && rm -rf /var/lib/apt/lists/*

RUN curl -L -o /usr/local/bin/websocat https://github.com/vi/websocat/releases/download/v1.14.0/websocat.x86_64-unknown-linux-musl && \
    chmod +x /usr/local/bin/websocat

RUN curl -L -o cardano-cli.tar.gz https://github.com/IntersectMBO/cardano-cli/releases/download/cardano-cli-10.1.1.0/cardano-cli-10.1.1.0-x86_64-linux.tar.gz && \
    tar -xzf cardano-cli.tar.gz && \
    mv cardano-cli-x86_64-linux /usr/local/bin/cardano-cli && \
    chmod +x /usr/local/bin/cardano-cli && \
    rm -rf cardano-cli.tar.gz cardano-cli-x86_64-linux

ARG TARGETARCH
COPY bin/proxy-Linux-${TARGETARCH} /hcp/proxy
COPY bin/operator-Linux-${TARGETARCH} /hcp/operator
COPY bin/rpc-Linux-${TARGETARCH} /hcp/rpc
COPY bin/metrics-exporter-Linux-${TARGETARCH} /hcp/metrics-exporter
COPY bin/open-head-Linux-${TARGETARCH} /hcp/open-head

RUN chmod +x /hcp/*

ENV PATH="/hcp:${PATH}"

# Configure rocket to listen on all interfaces.
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

# Expose the port that the application listens on.
EXPOSE 8000
