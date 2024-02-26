APP_NAME=muninn

main:
	cargo build --release

publish:
	sh scripts/publish_container.sh

pushpi:
	ssh heimdallr.local "mkdir -p ~/src/" && rsync -av --progress . heimdallr.local:~/src/$(APP_NAME)

install:
	# stop the service if it already exists
	systemctl stop muninn || true
	systemctl disable muninn || true
	# delete the old service file if it exists
	rm /etc/systemd/system/muninn.service || true
	cp scripts/muninn.service /etc/systemd/system/
