# syntax=docker/dockerfile:1.5.2

FROM gosh-ubuntu:22.04

WORKDIR /workdir

## downloads only the state of one commit into current dir (without history or any extra information)
RUN gosh-get commit gosh://0:8b1cbcd8b08a6c384e0db0d3513898d36203fced3e141a7f6b99cc580738fc22/awnion/aoc2022 HEAD

RUN gosh-get file gosh://0:8b1cbcd8b08a6c384e0db0d3513898d36203fced3e141a7f6b99cc580738fc22/awnion/aoc2022 HEAD LICENSE
RUN cat LICENSE | head
