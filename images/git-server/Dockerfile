# syntax=docker/dockerfile:1.5.2

FROM rust:1.70-bookworm as rust-builder

RUN apt-get update && apt-get install -yq \
    build-essential \
    protobuf-compiler

FROM rust-builder as gosh-git-server-build

WORKDIR /workdir
RUN \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/target \
    --mount=type=bind,target=./ \
    cargo install \
    --path ./git-server/ \
    --target-dir=/target

FROM ubuntu:22.04

RUN apt update && apt install -y ca-certificates git wget libssl3 libc6 curl

RUN wget -O - https://raw.githubusercontent.com/gosh-sh/gosh/dev/install.sh | bash -s \
    && mv /root/.gosh/git-remote-gosh* /usr/local/bin

COPY --link --from=gosh-git-server-build /usr/local/cargo/bin/gosh-git-server /usr/local/bin/gosh-git-server

EXPOSE 8080
CMD [ "gosh-git-server" ]
