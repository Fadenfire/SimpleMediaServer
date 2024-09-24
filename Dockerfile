# Use native platform for web-builder since it doesn't produce any platform specific artifacts
FROM --platform=$BUILDPLATFORM node:22.4.0-alpine AS web-builder

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

FROM rust:1.79.0-slim-bookworm AS builder

RUN rm -f /etc/apt/apt.conf.d/docker-clean && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && \
    apt-get --no-install-recommends install -y pkg-config clang cmake make nasm xz-utils

WORKDIR /app

ADD 'https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.0-latest-linux64-gpl-shared-7.0.tar.xz' ffmpeg.tar.xz
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
    sed -i "s/Components: main/Components: main non-free/" /etc/apt/sources.list.d/debian.sources && \
    apt-get update && \
    apt-get --no-install-recommends install -y xz-utils && \
    case "$(uname -m)" in \
    	x86_64|amd64) \
    		apt-get --no-install-recommends install -y intel-media-va-driver-non-free libva-drm2 libmfx1 ;; \
    	*) ;; \
    esac

WORKDIR /app

ADD 'https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.0-latest-linux64-gpl-shared-7.0.tar.xz' ffmpeg.tar.xz
RUN mkdir ffmpeg &&  \
    tar -xvf ffmpeg.tar.xz -C ffmpeg --strip-components=1 && \
    mv ffmpeg/bin/* /usr/local/bin && \
    mv ffmpeg/lib/* /usr/local/lib && \
    rm -rf ffmpeg/ ffmpeg.tar.xz && \
    ldconfig

COPY --from=builder /app/media-server media-server
COPY --from=web-builder /app/build/ web-ui

WORKDIR /data
ENTRYPOINT ["/app/media-server"]
CMD []
