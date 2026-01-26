# Compile le binaire Linux statique dans le conteneur
build:
	docker compose run --rm build-binary

push-s3:
	cp target/x86_64-unknown-linux-musl/release/auth-manager target/x86_64-unknown-linux-musl/release/bootstrap
	zip target/lambda.zip target/x86_64-unknown-linux-musl/release/bootstrap
	AWS_PROFILE=perso aws s3 cp target/lambda.zip s3://dev-rickycst-sandbox/lambda.zip

update-lambda:
	AWS_PROFILE=perso aws lambda update-function-code \
		--function-name testrust \
		--s3-bucket dev-rickycst-sandbox \
		--s3-key lambda.zip

COMPOSE_DEV = docker compose -f docker/docker-compose.yml
COMPOSE_TEST = $(COMPOSE_DEV) -f docker/docker-compose.test.yml

local:
	$(COMPOSE_DEV) up --build

migrate:
	$(COMPOSE_DEV) run --rm auth-manager diesel migration run

revert:
	$(COMPOSE_DEV) run --rm auth-manager diesel migration revert

test:
	$(COMPOSE_TEST) run --rm test-runner bash -c "diesel database setup && cargo test -- --test-threads=1"
	$(COMPOSE_TEST) stop auth-db-test
