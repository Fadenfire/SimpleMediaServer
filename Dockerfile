# ============= Build Frontend =============

# Use native platform for web-builder since it doesn't produce any platform specific artifacts
FROM --platform=$BUILDPLATFORM node:24.8.0-alpine AS web-builder

ENV COREPACK_HOME="/corepack"
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

COPY web-ui /app
WORKDIR /app

RUN --mount=type=cache,target=/corepack,sharing=locked \
    --mount=type=cache,target=/pnpm/store,sharing=locked \
    pnpm install --frozen-lockfile && \
    pnpm run build

# ============= Build Backend =============

FROM rust:1.89.0-slim-bookworm AS builder

RUN rm -f /etc/apt/apt.conf.d/docker-clean && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && \
    apt-get --no-install-recommends install -y pkg-config clang cmake make nasm xz-utils

WORKDIR /app

ADD 'https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2024-08-31-12-50/ffmpeg-n7.0.2-6-g7e69129d2f-linux64-gpl-shared-7.0.tar.xz' ffmpeg.tar.xz
RUN mkdir ffmpeg &&  \
    tar -xvf ffmpeg.tar.xz -C ffmpeg --strip-components=1 && \
    mv ffmpeg/bin/* /usr/local/bin && \
    mv ffmpeg/lib/* /usr/local/lib && \
    mv ffmpeg/include/* /usr/local/include && \
    rm -rf ffmpeg/ ffmpeg.tar.xz && \
    ldconfig

COPY Cargo.toml Cargo.lock ./
COPY src src
RUN --mount=type=cache,sharing=locked,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=locked,target=/app/target \
    cargo build --release && \
    cp target/release/simple-media-server ./

# ============= Final Container =============

FROM debian:bookworm-slim AS runtime

# Adapted from https://github.com/jellyfin/jellyfin/blob/9dd80083fbab61ed7af13e09906b76441c728bcb/Dockerfile#L36-L41
#  and https://github.com/jellyfin/jellyfin-packaging/blob/df6c5f8f5d4538f9e0f38003a1429c3419cacdcc/docker/Dockerfile#L144-L160
RUN rm -f /etc/apt/apt.conf.d/docker-clean && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && \
    apt-get install --no-install-recommends --no-install-suggests -y ca-certificates gnupg curl && \
    curl -fsSL https://repo.jellyfin.org/jellyfin_team.gpg.key | gpg --dearmor -o /etc/apt/trusted.gpg.d/debian-jellyfin.gpg && \
    echo "deb [arch=$( dpkg --print-architecture )] https://repo.jellyfin.org/master/debian $( awk -F'=' '/^VERSION_CODENAME=/{ print $NF }' /etc/os-release ) main" > /etc/apt/sources.list.d/jellyfin.list && \
    apt-get update && \
    apt-get install --no-install-recommends --no-install-suggests -y mesa-va-drivers jellyfin-ffmpeg7 && \
    apt-get remove gnupg -y

WORKDIR /app

COPY --from=builder /app/simple-media-server simple-media-server
COPY --from=web-builder /app/build/ web-ui

ENV LD_LIBRARY_PATH="/usr/lib/jellyfin-ffmpeg/lib/"

WORKDIR /data
ENTRYPOINT ["/app/simple-media-server"]
CMD []
