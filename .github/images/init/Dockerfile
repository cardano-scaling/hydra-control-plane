FROM amazon/aws-cli
RUN yum update -y  && yum install -y tar gzip unzip curl

WORKDIR /var/hydra-node
RUN curl -L -O https://github.com/cardano-scaling/hydra/releases/download/0.19.0/hydra-x86_64-linux-0.19.0.zip
RUN unzip -d bin hydra-x86_64-linux-0.19.0.zip
RUN cp /var/hydra-node/bin/hydra-node /hydra-node
RUN chmod +x /hydra-node

COPY .github/images/init/entrypoint.sh /entrypoint.sh
ENTRYPOINT ["sh", "/entrypoint.sh"]
