#!/usr/bin/make

.DEFAULT_GOAL := help

IMAGE_NAME := berne/dark-privilege
RELEASE := latest

.PHONY:	help
help:
	@grep -E '^[a-zA-Z_-]+([a-zA-Z_ -.])*:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1,$$2}'

.PHONY:	_docker-image docker-image docker build
docker:	docker-image
build:  docker-image
docker-image:		## Build docker image
	@echo "Builing ${IMAGE_NAME}:${RELEASE}..."
	docker build \
	  -t ${IMAGE_NAME}:${RELEASE} \
	  --provenance true \
	  .

dark-privilege:	docker-image 	## Linux executable
	docker create --name dark-privilege $(IMAGE_NAME)
	docker cp dark-privilege:/dark-privilege .
	docker rm dark-privilege

dark-privilege.exe:	docker-image	## Windows GNU executable
	docker create --name dark-privilege $(IMAGE_NAME)
	docker cp dark-privilege:/x86_64-pc-windows-gnu/release/dark-privilege.exe .
	docker rm dark-privilege

.PHONY:	dockerfile-lint hadolint
dockerfile-lint:	hadolint	## Run linter on Dockerfile
hadolint:
	docker run --rm hadolint/hadolint < Dockerfile

.PHONY:	pre-commit-install
pre-commit-install:	## Setup pre-commit git hooks
	pre-commit install

.PHONY:	pre-commit-run-allow
pre-commit-run-all:	## Run pre-commit checks
	pre-commit run --all

