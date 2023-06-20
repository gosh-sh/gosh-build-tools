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
	GOSH_LOG=info,gosh_builder=debug cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q -c hack/Gosh.yaml

.PHONY: debug-url
debug-url: gosh-ubuntu
	GOSH_LOG=info,gosh_builder=debug cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q gosh://0:0d5c05d7a63f438b57ede179b7110d3e903f5be3b5f543d3d6743d774698e92c/awnion/telepresence-gosh

.PHONY: trace
trace: gosh-ubuntu
	GOSH_LOG=info,gosh_builder=trace cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q -c hack/Gosh.yaml

.PHONY: trace-url
trace-url: gosh-ubuntu
	GOSH_LOG=info,gosh_builder=trace cargo run --bin gosh -- build -s ${PROXY_IP}:${PROXY_PORT} -q gosh://0:0d5c05d7a63f438b57ede179b7110d3e903f5be3b5f543d3d6743d774698e92c/awnion/telepresence-gosh

.PHONY: gosh-ubuntu
gosh-ubuntu: pb
	docker buildx build \
		--build-arg BRANCH=dev \
		--tag gosh-ubuntu \
		--file images/ubuntu/Dockerfile \
		.

.PHONY: gosh-rust
gosh-rust: pb
	docker buildx build \
		--tag gosh-rust \
		--file images/rust/Dockerfile \
		.

.PHONY: gosh-ubuntu-push
gosh-ubuntu-push: pb
	docker buildx build \
		--no-cache \
		--build-arg BRANCH=dev \
		--tag awnion/gosh-ubuntu \
		--tag awnion/gosh-ubuntu:22.04 \
		--file images/ubuntu/Dockerfile \
		--push \
		.

.PHONY: gosh-git-server
gosh-git-server:
	docker buildx build \
		--tag gosh-git-server \
		--file images/git-server/Dockerfile \
		.

.PHONY: gosh-git-server
gosh-git-server-debug: gosh-git-server
	docker run --rm -ti -p 8080:8080 -e GOSH_LOG=debug gosh-git-server

.PHONY: gosh-git-server-push
gosh-git-server-push:
	docker buildx build \
		--tag awnion/gosh-git-server \
		--file images/git-server/Dockerfile \
		--push \
		.

.PHONY: gosh-rust-push
gosh-rust-push: pb
	docker buildx build \
		--no-cache \
		--tag awnion/gosh-rust \
		--file images/rust/Dockerfile \
		--push \
		.

.PHONY: gosh-ubuntu-release
gosh-ubuntu-release: pb
	docker buildx build \
		--no-cache \
		--build-arg BRANCH=dev \
		--tag teamgosh/gosh-ubuntu:22.04 \
		--tag teamgosh/gosh-ubuntu:latest \
		--file images/ubuntu/Dockerfile \
		--push \
		.

.PHONY: gosh-rust-release
gosh-rust-release: pb
	docker buildx build \
		--no-cache \
		--tag teamgosh/gosh-rust:1.70-bookworm \
		--tag teamgosh/gosh-rust:latest \
		--file images/rust/Dockerfile \
		--push \
		.

.PHONY: gosh-golang-release
gosh-golang-release: pb
	docker buildx build \
		--no-cache \
		--tag teamgosh/gosh-golang:1.20.4-bullseye \
		--tag teamgosh/gosh-golang:latest \
		--file images/go/Dockerfile \
		--push \
		.

.PHONY: pb
pb:
	cd gosh-builder-grpc-api && cargo build

.PHONY: clear
clear:
	rm -rf ./sbom.*

.PHONY: fmt
fmt:
	taplo fmt
	cargo fmt --all

.PHONY: init
init:
	cargo run --bin gosh init

.PHONY: install
install:
	cd gosh && cargo install -f --path .
	cd git-server && cargo install -f --path .

.PHONY: dev-install ## fast builds for debug
dev-install:
	cd gosh && cargo install --profile dev -f --path .
	cd git-server && cargo install --profile dev -f --path .
