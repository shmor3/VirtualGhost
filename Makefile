DOCKER_IMAGE := virtualghost-builder
DOCKER_TAG   := latest
ASSETS_DIR   := assets

.PHONY: assets docker-build guest-agent clean-assets

# Build kernel + rootfs via Docker, output to assets/
assets: docker-build
	docker run --rm --privileged \
		-v "$$(pwd)/assets:/output" \
		-v "$$(pwd)/guest:/build/guest" \
		$(DOCKER_IMAGE):$(DOCKER_TAG)

# Build the Docker builder image
docker-build:
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) -f guest/build/Dockerfile guest/build/

# Build guest agent natively (Linux only)
guest-agent:
	cargo build --manifest-path guest/ghostly-agent/Cargo.toml \
		--target x86_64-unknown-linux-musl --release

# Remove built assets
clean-assets:
	rm -f $(ASSETS_DIR)/vmlinux $(ASSETS_DIR)/rootfs.ext4
