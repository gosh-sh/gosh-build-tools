# syntax=docker/dockerfile:1.5.2

FROM gosh-ubuntu
ARG GOSH_HTTP_PROXY
ENV GOSH_HTTP_PROXY=${GOSH_HTTP_PROXY}

WORKDIR /workdir

RUN git clone \
    $GOSH_HTTP_PROXY/0:8b1cbcd8b08a6c384e0db0d3513898d36203fced3e141a7f6b99cc580738fc22/awnion/aoc2022 \
    .

RUN cat LICENSE | head
