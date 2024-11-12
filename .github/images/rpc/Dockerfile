ARG RUST_VERSION=1.81.0
ARG BIN=hydra-control-plane
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG BIN
WORKDIR /app

RUN apt-get update && apt-get install -y libssl-dev pkg-config

# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies and a cache mount to /app/target/ for 
# compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$BIN /bin/program
EOF

FROM debian:bullseye-slim AS final

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/develop/develop-images/dockerfile_best-practices/#user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/home/app" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    appuser
USER appuser
WORKDIR /home/app

# Copy the executable from the "build" stage.
COPY --from=build /bin/program /bin/program
COPY Rocket.toml /Rocket.toml

# Configure rocket to listen on all interfaces.
ENV ROCKET_ADDRESS=0.0.0.0

# Expose the port that the application listens on.
EXPOSE 8000

# What the container should run when it is started.
CMD ["/bin/program"]
