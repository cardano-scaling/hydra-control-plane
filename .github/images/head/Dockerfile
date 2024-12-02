FROM ghcr.io/cardano-scaling/hydra-control-plane:sha-a9c3b67

RUN apt-get update && apt-get install -y curl jq unzip

WORKDIR /var/hydra-node
RUN curl -L -O https://github.com/cardano-scaling/hydra/releases/download/0.19.0/hydra-x86_64-linux-0.19.0.zip
RUN unzip -d bin hydra-x86_64-linux-0.19.0.zip
RUN cp /var/hydra-node/bin/hydra-node /hydra-node
RUN chmod +x /hydra-node
