.DEFAULT_GOAL := help

PROXY_IP ?= 127.0.0.1
PROXY_PORT ?= 6060

.PHONY: run
run: gosh-ubuntu
	cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q -c hack/Gosh.yaml

.PHONY: run-url
run-url: gosh-ubuntu
	cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q gosh://0:0d5c05d7a63f438b57ede179b7110d3e903f5be3b5f543d3d6743d774698e92c/awnion/telepresence-gosh

.PHONY: debug
debug: gosh-ubuntu
	RUST_LOG=info,gosh_builder_cli=debug cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q -c hack/Gosh.yaml

.PHONY: trace
trace: gosh-ubuntu
	RUST_LOG=info,gosh_builder_cli=trace cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q -c hack/Gosh.yaml

.PHONY: gosh-ubuntu
gosh-ubuntu: pb
	docker buildx build \
		` # --progress=plain ` \
		--build-arg BRANCH=dev \
		--tag gosh-ubuntu \
		--file images/Dockerfile \
		.

# TODO: make multiplatform build
.PHONY: gosh-ubuntu-push
gosh-ubuntu-push: pb
	docker buildx build \
		` # --progress=plain ` \
		--no-cache \
		--build-arg BRANCH=dev \
		--tag awnion/gosh-ubuntu:22.04 \
		--file images/Dockerfile \
		--push \
		.

.PHONY: pb
pb:
	cd gosh-builder-grpc-api && cargo build

.PHONY: clear
clear:
	rm -rf ./sbom.*

.PHONY: init
init:
	cargo run --bin gosh init

.PHONY: install
install:
	cd gosh && cargo install -f --path .

.PHONY: install-builder
install-builder: install
	cd gosh-builder-cli && cargo install -f --path .
