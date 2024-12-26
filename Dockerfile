# Use native platform for web-builder since it doesn't produce any platform specific artifacts
FROM --platform=$BUILDPLATFORM node:23.3.0-alpine AS web-builder

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

FROM rust:1.82.0-slim-bookworm AS builder

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
    cp target/release/media-server ./

FROM debian:bookworm-slim AS runtime

RUN rm -f /etc/apt/apt.conf.d/docker-clean && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && \
    apt-get install --no-install-recommends --no-install-suggests -y ca-certificates gnupg curl && \
    curl -fsSL https://repo.jellyfin.org/jellyfin_team.gpg.key | gpg --dearmor -o /etc/apt/trusted.gpg.d/debian-jellyfin.gpg && \
    echo "deb [arch=$( dpkg --print-architecture )] https://repo.jellyfin.org/$( awk -F'=' '/^ID=/{ print $NF }' /etc/os-release ) $( awk -F'=' '/^VERSION_CODENAME=/{ print $NF }' /etc/os-release ) main" | tee /etc/apt/sources.list.d/jellyfin.list && \
    apt-get update && \
    apt-get install --no-install-recommends --no-install-suggests -y mesa-va-drivers jellyfin-ffmpeg7 && \
    apt-get remove gnupg -y

WORKDIR /app

COPY --from=builder /app/media-server media-server
COPY --from=web-builder /app/build/ web-ui

ENV LD_LIBRARY_PATH="/usr/lib/jellyfin-ffmpeg/lib/"

WORKDIR /data
ENTRYPOINT ["/app/media-server"]
CMD []
