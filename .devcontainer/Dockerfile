FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

USER root
RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates \
        git \
        pkg-config \
        libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*
USER vscode
