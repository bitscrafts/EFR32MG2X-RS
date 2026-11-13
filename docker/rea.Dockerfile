# A reproducible image for reading vendor blobs.
#
# Why an image at all: the toolchain that reads the blobs is version-locked in three places (Node 22+ for
# rea-agents, JDK 25 for Ghidra 12.2, Ghidra 12.x for REA's bridge) and the container this project runs in
# ships a Node that rea-agents refuses. Pinning them here means the next person does not rediscover that.
#
# Build and use:
#
#   podman build -f docker/rea.Dockerfile -t efr32-rea .
#   podman run --rm -v /projects:/projects:ro efr32-rea \
#     node /opt/rea/node_modules/rea-agents/scripts/rea.mjs analyze \
#       /projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/rail_ble_tx --format md
#
# The mount is read-only: this image exists to read artefacts, never to modify them.

FROM debian:13-slim

ARG GHIDRA_VER=ghidra_12.2_PUBLIC
ARG GHIDRA_ZIP=ghidra_12.2_PUBLIC_20261016.zip
ARG GHIDRA_URL=https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_12.2_build/${GHIDRA_ZIP}
ARG NODE_CHANNEL=22

ENV DEBIAN_FRONTEND=noninteractive \
    JAVA_HOME=/usr/lib/jvm/java-25-openjdk-amd64 \
    GHIDRA_INSTALL_DIR=/opt/ghidra/${GHIDRA_VER}

# JDK 25 is a hard requirement of Ghidra 12.2; 21 fails with "unsupported_version".
RUN apt-get update -qq \
 && apt-get install -y -qq --no-install-recommends \
      openjdk-25-jdk-headless curl ca-certificates python3 xz-utils binutils-arm-none-eabi \
 && rm -rf /var/lib/apt/lists/*

# Node 22 for rea-agents (the distro Node is too old or, as in the dev container, too *new*).
RUN curl -fsSL "https://deb.nodesource.com/setup_${NODE_CHANNEL}.x" | bash - \
 && apt-get install -y -qq --no-install-recommends nodejs \
 && rm -rf /var/lib/apt/lists/* \
 && node --version

# Ghidra, extracted with `python3 -m zipfile`, which preserves the executable bit. A plain
# zipfile.extractall() leaves analyzeHeadless non-executable and REA then reports "executable_missing"
# about a file that is plainly there.
RUN mkdir -p /opt/ghidra && cd /opt/ghidra \
 && curl -fsSL -o "${GHIDRA_ZIP}" "${GHIDRA_URL}" \
 && python3 -m zipfile -e "${GHIDRA_ZIP}" . \
 && rm -f "${GHIDRA_ZIP}" \
 && chmod +x "${GHIDRA_VER}"/support/analyzeHeadless "${GHIDRA_VER}"/support/*.sh \
             "${GHIDRA_VER}"/ghidraRun "${GHIDRA_VER}"/Ghidra/Features/Decompiler/os/linux_x86_64/* \
 && test -x "${GHIDRA_INSTALL_DIR}/support/analyzeHeadless"

RUN mkdir -p /opt/rea && cd /opt/rea \
 && npm init -y >/dev/null \
 && npm i rea-agents@latest

# The verification that matters: REA must report the Ghidra provider as available, which proves the three
# versions were actually compatible rather than merely installed.
RUN node /opt/rea/node_modules/rea-agents/scripts/rea.mjs capabilities \
      | grep -A3 "id: ghidra" | grep -q "status: available" \
 && echo "REA + Ghidra provider: available"

WORKDIR /projects
ENTRYPOINT ["node", "/opt/rea/node_modules/rea-agents/scripts/rea.mjs"]
CMD ["--help"]
