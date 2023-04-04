.DEFAULT_GOAL := help

PROXY_IP ?= 127.0.0.1
PROXY_PORT ?= 8000

.PHONY: run
run:
	cargo run -- --config hack/Gosh.yaml

.PHONY: gosh-ubuntu
gosh-ubuntu:
	cd images && docker buildx build \
		--progress=plain \
		--no-cache \
		--build-arg BRANCH=release-3.0.17 \
		--tag gosh-ubuntu:22.04 \
		.

.PHONY: clear
clear:
	rm -rf ./sbom.*
	rm -rf ./.git-cache
