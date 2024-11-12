FROM debian:12-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
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
