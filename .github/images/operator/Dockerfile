FROM debian:12-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY bin/operator-Linux-${TARGETARCH} /bin/operator
RUN chmod +x /bin/operator
RUN ln -s /bin/operator /operator

ENTRYPOINT [ "operator" ]
