FROM archlinux:latest

# Install build dependencies
RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm \
    base-devel \
    arch-install-scripts \
    e2fsprogs \
    libarchive \
    rustup \
    musl && \
    pacman -Scc --noconfirm

# Install Rust toolchain with musl target
RUN rustup default stable && \
    rustup target add x86_64-unknown-linux-musl

# Copy build scripts to a fixed location (not under /build/guest which gets volume-mounted)
COPY build-all.sh build-kernel.sh build-rootfs.sh /opt/builder/
COPY rootfs/ /opt/builder/rootfs/
RUN chmod +x /opt/builder/*.sh

WORKDIR /build

ENTRYPOINT ["/opt/builder/build-all.sh"]
