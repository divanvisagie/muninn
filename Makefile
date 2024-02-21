main:
	cargo build --release

publish:
	sh scripts/publish_container.sh