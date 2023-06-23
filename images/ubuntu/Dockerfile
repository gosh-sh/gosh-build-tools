# syntax=docker/dockerfile:1.5.2

FROM ubuntu:22.04

RUN apt update \
    && apt install -qy jq ca-certificates git wget libssl3 libc6 curl netcat

RUN wget -O - https://raw.githubusercontent.com/gosh-sh/gosh/dev/install.sh | bash -s

ONBUILD ARG GOSH_HTTP_PROXY
ONBUILD ENV GOSH_HTTP_PROXY=$GOSH_HTTP_PROXY
