.DEFAULT_GOAL := help

PROXY_IP ?= 127.0.0.1
PROXY_PORT ?= 6060

.PHONY: run
run: gosh-ubuntu
	cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q -c hack/Gosh.yaml

.PHONY: run-fail
run-fail: gosh-ubuntu
	cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q -c hack/Gosh.fail_test.yaml

.PHONY: run-url
run-url: gosh-ubuntu
	cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q gosh://0:0d5c05d7a63f438b57ede179b7110d3e903f5be3b5f543d3d6743d774698e92c/awnion/telepresence-gosh

.PHONY: debug
debug: gosh-ubuntu
	RUST_LOG=info,gosh_builder=debug cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q -c hack/Gosh.yaml

.PHONY: debug-url
debug-url: gosh-ubuntu
	RUST_LOG=info,gosh_builder=debug cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q gosh://0:0d5c05d7a63f438b57ede179b7110d3e903f5be3b5f543d3d6743d774698e92c/awnion/telepresence-gosh

.PHONY: trace
trace: gosh-ubuntu
	RUST_LOG=info,gosh_builder=trace cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q -c hack/Gosh.yaml

.PHONY: trace-url
trace-url: gosh-ubuntu
	RUST_LOG=info,gosh_builder=trace cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q gosh://0:0d5c05d7a63f438b57ede179b7110d3e903f5be3b5f543d3d6743d774698e92c/awnion/telepresence-gosh

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

gosh-ubuntu-release: pb
	docker buildx build \
		` # --progress=plain ` \
		--no-cache \
		--build-arg BRANCH=dev \
		--tag teamgosh/gosh-ubuntu:22.04 \
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
	cd telepresence-build-gosh && cargo install -f --path .
