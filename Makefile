.DEFAULT_GOAL := help

PROXY_IP ?= 127.0.0.1
PROXY_PORT ?= 8000

.PHONY: run
run:
	@echo Proxy IP $(PROXY_IP) PORT $(PROXY_PORT)
	cargo run -- --config hack/Gosh.yaml

.PHONY: gosh-ubuntu
gosh-ubuntu:
	cd images && docker buildx build \
		--progress=plain \
		--tag gosh-ubuntu:22.04 \
		.

.PHONY: gosh-grpc-client
gosh-grpc-client:
	docker buildx build \
		--progress=plain \
		--tag gosh-grpc-client \
		-f gosh-grpc-cli/Dockerfile \
		.
