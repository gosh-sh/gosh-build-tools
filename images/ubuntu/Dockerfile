# syntax=docker/dockerfile:1.5.2

FROM rust:1.70-bookworm as rust-builder

RUN apt-get update && apt-get install -yq \
    build-essential \
    protobuf-compiler

FROM rust-builder as gosh-get-build

WORKDIR /workdir
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/target \
    --mount=type=bind,target=./ \
    cargo install \
    --path ./gosh-get/ \
    --target-dir=/target

FROM ubuntu:22.04

RUN apt update && apt install -y jq ca-certificates git wget libssl3 libc6 curl netcat

RUN wget -O - https://raw.githubusercontent.com/gosh-sh/gosh/dev/install.sh | bash -s

COPY --link --from=gosh-get-build /usr/local/cargo/bin/gosh-get /usr/local/bin/gosh-get

ONBUILD ARG GOSH_HTTP_PROXY
ONBUILD ENV GOSH_HTTP_PROXY=$GOSH_HTTP_PROXY
