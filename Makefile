default: run

.PHONY: build
build:
	wrangler build

.PHONY: run
run:
	miniflare -w -e .env

.PHONY: deploy
deploy:
	wrangler publish

.PHONY: trigger
trigger:
	twitch event trigger -s twitch_secret subscribe -F http://localhost:8787/callback

.PHONY: docker-watch
docker-watch:
	docker build -t taniwha3/tilkynningu .
	docker run --net=host --volume $(PWD):/workdir -it taniwha3/tilkynningu
