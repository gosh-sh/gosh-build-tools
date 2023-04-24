# syntax=docker/dockerfile:1.5.2

FROM gosh-ubuntu:22.04

ENV GOSH_GRPC_ENABLE=1

WORKDIR /workdir

RUN git clone --depth 1 gosh://0:8b1cbcd8b08a6c384e0db0d3513898d36203fced3e141a7f6b99cc580738fc22/awnion/_index
RUN git clone --depth 1 gosh://0:8b1cbcd8b08a6c384e0db0d3513898d36203fced3e141a7f6b99cc580738fc22/awnion/aoc2022
