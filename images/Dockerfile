# syntax=docker/dockerfile:1.5.2

FROM ubuntu:22.04

RUN apt update && apt install -y jq ca-certificates git wget libssl3 libc6 curl

ARG BRANCH=dev
ENV BRANCH=${BRANCH}
RUN <<EOF
    set -ex
    echo "$BRANCH"
    git clone --branch "$BRANCH" --depth 1 https://github.com/gosh-sh/gosh.git gosh
    cd gosh
    bash install.sh
    mv $HOME/.gosh/git-remote-gosh /usr/local/bin/git-remote-gosh
    mv $HOME/.gosh/git-remote-gosh_v1_0_0 /usr/local/bin/git-remote-gosh_v1_0_0
    mv $HOME/.gosh/git-remote-gosh_v2_0_0 /usr/local/bin/git-remote-gosh_v2_0_0
    cd ..
    rm -rf gosh
EOF
