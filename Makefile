APP_NAME=muninn

main:
	cargo build --release

publish:
	sh scripts/publish_container.sh

pushpi:
	ssh heimdallr.local "mkdir -p ~/src/" && rsync -av --progress . heimdallr.local:~/src/$(APP_NAME)
