# syntax=docker/dockerfile:1.5.2

FROM gosh-ubuntu:22.04

ENV GOSH_GRPC_ENABLE=1

WORKDIR /workdir

# downloads only the state of one commit into current dir (without history or any extra information)
RUN gosh-get commit gosh://0:8b1cbcd8b08a6c384e0db0d3513898d36203fced3e141a7f6b99cc580738fc22/awnion/aoc2022 HEAD
RUN cat LICENSE | head

# NOTE:
# alternatively you can use "good old" `git`
# BUT in case of git it's not guarantied that you will get only one commit (due
# to git architecture). Even if you set `--depth 1` some other commits might be in
# the store due to some weired branch merges and optimizations

# RUN git clone --depth 1 gosh://0:8b1cbcd8b08a6c384e0db0d3513898d36203fced3e141a7f6b99cc580738fc22/awnion/_index
# RUN git clone --depth 1 gosh://0:8b1cbcd8b08a6c384e0db0d3513898d36203fced3e141a7f6b99cc580738fc22/awnion/aoc2022
