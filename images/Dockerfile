# syntax=docker/dockerfile:1.5.2

FROM rust:1.69-bookworm as rust-builder

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

RUN apt update && apt install -y jq ca-certificates git wget libssl3 libc6 curl

RUN wget -O - https://raw.githubusercontent.com/gosh-sh/gosh/dev/install.sh | bash -s

# ARG BRANCH=dev
# ENV BRANCH=${BRANCH}
# RUN <<EOF
#     set -ex
#     echo "$BRANCH"
#     git clone --branch "$BRANCH" --depth 1 https://github.com/gosh-sh/gosh.git gosh
#     cd gosh
#     bash install.sh
#     echo '{
#       "primary-network": "mainnet",
#       "networks": {
#         "mainnet": {
#           "user-wallet": {
#             "profile": "tester9999",
#             "pubkey": "7930e4c68e89afb613ee060c47e34dfbbb311b8374cce7d901b34276ad7c8e62",
#             "secret": "367f512b6a329d56943e60a3bb10794b0f3fe15c05e159a9b5a21c8df7b72be7"
#           },
#           "endpoints": [
#             "https://bhs01.network.gosh.sh",
#             "https://eri01.network.gosh.sh",
#             "https://gra01.network.gosh.sh"
#           ]
#         }
#       }
#     }' > $HOME/.gosh/config.json
#     mv $HOME/.gosh/git-remote-gosh /usr/local/bin/git-remote-gosh
#     mv $HOME/.gosh/git-remote-gosh_v1_0_0 /usr/local/bin/git-remote-gosh_v1_0_0
#     mv $HOME/.gosh/git-remote-gosh_v2_0_0 /usr/local/bin/git-remote-gosh_v2_0_0
#     mv $HOME/.gosh/git-remote-gosh_v3_0_0 /usr/local/bin/git-remote-gosh_v3_0_0
#     cd ..
#     rm -rf gosh
#     cd /tmp
#     mkdir test
#     cd test
#     git init
# EOF

COPY --link --from=gosh-get-build /usr/local/cargo/bin/gosh-get /usr/local/bin/gosh-get
