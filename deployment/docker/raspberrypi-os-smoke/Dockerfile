FROM dtcooper/raspberrypi-os:bookworm
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates systemd alsa-utils && rm -rf /var/lib/apt/lists/*
COPY deployment/docker/raspberrypi-os-smoke/run.sh /usr/local/bin/openiem-arm64-smoke
ENTRYPOINT ["/usr/local/bin/openiem-arm64-smoke"]
