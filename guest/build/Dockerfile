FROM archlinux:latest

# Update and install build dependencies
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm \
    # Kernel extraction
    base-devel \
    # Rootfs creation
    arch-install-scripts e2fsprogs \
    # Guest agent cross-compilation
    rust musl \
    # Utilities
    bsdtar && \
    # Add musl target for Rust
    rustup target add x86_64-unknown-linux-musl && \
    # Clean pacman cache
    pacman -Scc --noconfirm

WORKDIR /build

COPY build-all.sh build-kernel.sh build-rootfs.sh /build/guest/build/
COPY rootfs/ /build/guest/build/rootfs/
RUN chmod +x /build/guest/build/*.sh

ENTRYPOINT ["/build/guest/build/build-all.sh"]
