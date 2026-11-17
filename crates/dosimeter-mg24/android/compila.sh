#!/usr/bin/env bash
# compila.sh — compila o APK do dosímetro dentro do container, de forma re-executável.
#
# ## Porque é um script
#
# A primeira tentativa foi uma linha de comando que descarregava o Gradle dentro do
# container. O container é efémero (`--rm`): o download desapareceu com ele e a segunda
# invocação falhou com `gradle: command not found`. Um build que depende de estado
# efémero não é re-executável — este script monta a cache em disco e por isso pode
# correr duas vezes seguidas com o mesmo resultado.
#
# ## O que persiste
#
#   .gradle-dist/    a distribuição do Gradle (130 MB)
#   .gradle-home/    dependências Maven do AGP e do Kotlin
#
# Uso:
#   bash android/compila.sh            # compila
#   bash android/compila.sh limpa      # apaga as caches (obriga a novo download)
set -euo pipefail

AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGEM="docker.io/mingc/android-build-box:latest"
GRADLE_VER="8.7"

if [[ "${1:-}" == "limpa" ]]; then
    echo "a remover caches: ${AQUI}/.gradle-dist ${AQUI}/.gradle-home"
    mv "${AQUI}/.gradle-dist" "${AQUI}/.gradle-dist.old-$(date +%s)" 2>/dev/null || true
    mv "${AQUI}/.gradle-home" "${AQUI}/.gradle-home.old-$(date +%s)" 2>/dev/null || true
    exit 0
fi

mkdir -p "${AQUI}/.gradle-dist" "${AQUI}/.gradle-home"

REMOTO="/podman/assistants/hermes/projects/efr32mg24-rs/crates/dosimeter-mg24/android"

ssh mvcorrea@lnx "podman run --rm \
  -v '${REMOTO}':/project \
  -v '${REMOTO}/.gradle-dist':/opt/gradle \
  -v '${REMOTO}/.gradle-home':/root/.gradle \
  -w /project \
  ${IMAGEM} bash -lc '
    set -e
    GRADLE=/opt/gradle/gradle-${GRADLE_VER}/bin/gradle
    if [ ! -x \"\$GRADLE\" ]; then
      echo \"a descarregar gradle ${GRADLE_VER} (uma vez)\"
      curl -sL -o /tmp/g.zip https://services.gradle.org/distributions/gradle-${GRADLE_VER}-bin.zip
      unzip -q /tmp/g.zip -d /opt/gradle
    fi
    \"\$GRADLE\" assembleDebug --no-daemon --console=plain 2>&1 | tail -18
  '"
