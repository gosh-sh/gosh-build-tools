.DEFAULT_GOAL := help

PROXY_IP ?= 127.0.0.1
PROXY_PORT ?= 8000

.PHONY: run
run: gosh-ubuntu
	cargo run --bin gosh-builder-cli -- --config hack/Gosh.yaml

.PHONY: debug
debug: gosh-ubuntu
	RUST_LOG=info,gosh_builder_cli=debug cargo run --bin gosh-builder-cli -- --config hack/Gosh.yaml

.PHONY: trace
trace: gosh-ubuntu
	RUST_LOG=info,gosh_builder_cli=trace run --bin gosh-builder-cli -- --config hack/Gosh.yaml

.PHONY: gosh-ubuntu
gosh-ubuntu: pb
	docker buildx build \
		` # --progress=plain ` \
		--build-arg BRANCH=dev \
		--tag gosh-ubuntu:22.04 \
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
