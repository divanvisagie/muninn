APP_NAME=muninn

main:
	cargo build --release

publish:
	sh scripts/publish_container.sh

pushpi:
	ssh $(PI) "mkdir -p ~/src/" \
	&& rsync -av --progress src $(PI):~/src/$(APP_NAME) \
    && rsync -av --progress Cargo.toml $(PI):~/src/$(APP_NAME) \
	&& rsync -av --progress Cargo.lock $(PI):~/src/$(APP_NAME) \
	&& rsync -av --progress Makefile $(PI):~/src/$(APP_NAME) \

install:
	# stop the service if it already exists
	systemctl stop muninn || true
	systemctl disable muninn || true
	# delete the old service file if it exists
	rm /etc/systemd/system/muninn.service || true
	cp scripts/muninn.service /etc/systemd/system/

dev:
	cargo watch -x run
