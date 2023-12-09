# include .env file and export its env vars
# (-include to ignore error if it does not exist)
-include .env

.PHONY: build publish

# Variables
DOCKERHUB_ORGANIZATION ?= switchboardlabs

check_docker_env:
ifeq ($(strip $(DOCKERHUB_ORGANIZATION)),)
	$(error DOCKERHUB_ORGANIZATION is not set)
else
	@echo DOCKERHUB_ORGANIZATION: ${DOCKERHUB_ORGANIZATION}
endif

# Default make task
all: anchor_sync build

anchor_sync :; anchor keys sync
anchor_build :; anchor build

build: anchor_build docker_build measurement

build-basic-function: check_docker_env
	docker buildx build --pull --platform linux/amd64 \
		-f ./switchboard-functions/02_usdy_usdc_oracle_function_rust/Dockerfile \
		-t ${DOCKERHUB_ORGANIZATION}/solana-ondo-oracle-function:latest \
		./

publish-basic-function: check_docker_env
	docker buildx build --pull --platform linux/amd64 \
		-f ./switchboard-functions/02_usdy_usdc_oracle_function_rust/Dockerfile \
		-t ${DOCKERHUB_ORGANIZATION}/solana-ondo-oracle-function:latest \
		--push \
		./

build: docker_build measurement

publish: build-basic-function measurement

measurement: check_docker_env
	@docker run -d --platform=linux/amd64 -q --name=my-switchboard-function \
		${DOCKERHUB_ORGANIZATION}/solana-ondo-oracle-function:latest > /dev/null
	@docker cp my-switchboard-function:/measurement.txt measurement.txt
	@echo -n 'MrEnclve: '
	@cat measurement.txt
	@docker stop my-switchboard-function > /dev/null
	@docker rm my-switchboard-function > /dev/null

docker_build: check_docker_env
	docker buildx build --pull --platform linux/amd64 \
		-f ./switchboard-functions/02_usdy_usdc_oracle_function_rust/Dockerfile \
		-t ${DOCKERHUB_ORGANIZATION}/solana-ondo-oracle-function:latest \
		./switchboard-functions/02_usdy_usdc_oracle_function_rust
docker_publish: check_docker_env
	docker buildx build --pull --platform linux/amd64 \
		-f ./switchboard-functions/02_usdy_usdc_oracle_function_rust/Dockerfile \
		-t ${DOCKERHUB_ORGANIZATION}/solana-ondo-oracle-function:latest \
		--push \
		./switchboard-functions/02_usdy_usdc_oracle_function_rust
