FROM docker.io/rust:1-slim-bookworm AS build

## cargo package name: customize here or provide via --build-arg
ARG pkg=buildingstore-be

WORKDIR /build

COPY . .

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN --mount=type=cache,target=/build/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -eux; \
    cargo build --release; \
    objcopy --compress-debug-sections target/release/$pkg ./main

################################################################################

FROM docker.io/debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

## copy the main binary
COPY --from=build /build/main ./

## copy runtime assets which may or may not exist
COPY --from=build /build/Rocket.tom[l] ./static
COPY --from=build /build/stati[c] ./static
COPY --from=build /build/template[s] ./templates

## ensure the container listens globally on port 80
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80

EXPOSE 80

CMD ./main