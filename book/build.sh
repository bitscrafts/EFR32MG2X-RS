#!/usr/bin/env bash
# Build the book, the same way CI does.
#
#   bash book/build.sh              # local XeLaTeX if present, else the TeX Live image
#   bash book/build.sh --local      # force local XeLaTeX
#   bash book/build.sh --container  # force the image (podman or docker)
#   bash book/build.sh --no-copy    # leave the PDF in book/latex/ only
#
# Order matters and mirrors the workflow: check the sources, check that the printed code parses, check that the
# references point at something, compile twice so the contents and cross-references settle, then refuse an
# implausible page count. A book that builds while printing code nobody compiled is the failure this guards
# against, and it is silent -- the PDF looks fine.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(dirname "${HERE}")"
TEXLIVE_IMAGE="${TEXLIVE_IMAGE:-docker.io/texlive/texlive:latest}"
OUT_NAME="Rust-Embedded-Hardware-Support.pdf"

MODE="auto"
COPY=1
for arg in "$@"; do
    case "${arg}" in
        --local)     MODE="local" ;;
        --container) MODE="container" ;;
        --no-copy)   COPY=0 ;;
        *) echo "unknown option: ${arg}" >&2; exit 2 ;;
    esac
done

echo "=== checking the sources"
python3 "${HERE}/check-tex.py"
python3 "${HERE}/check-code-blocks.py"
python3 "${HERE}/check-refs.py"

compile_local() {
    echo "=== compiling with the local XeLaTeX (two passes)"
    cd "${HERE}/latex"
    xelatex -shell-escape -interaction=nonstopmode main.tex > pass1.log 2>&1 || true
    if ! xelatex -shell-escape -interaction=nonstopmode main.tex > pass2.log 2>&1; then
        echo "--- second pass failed:" >&2
        tail -40 pass2.log >&2
        exit 1
    fi
}

compile_container() {
    local engine
    if command -v podman >/dev/null 2>&1; then engine=podman
    elif command -v docker >/dev/null 2>&1; then engine=docker
    else
        echo "no local XeLaTeX and no podman/docker: install texlive-xetex or a container engine" >&2
        exit 1
    fi
    echo "=== compiling in ${TEXLIVE_IMAGE} via ${engine} (two passes)"
    "${engine}" run --rm -v "${HERE}":/work -w /work/latex "${TEXLIVE_IMAGE}" sh -c '
        xelatex -shell-escape -interaction=nonstopmode main.tex > pass1.log 2>&1 || true
        xelatex -shell-escape -interaction=nonstopmode main.tex > pass2.log 2>&1 || { tail -40 pass2.log; exit 1; }
        tail -2 pass2.log
    '
}

case "${MODE}" in
    local)     compile_local ;;
    container) compile_container ;;
    auto)
        if command -v xelatex >/dev/null 2>&1; then compile_local; else compile_container; fi
        ;;
esac

echo "=== checking the result"
PAGES=$(sed -n 's/.*Output written on main\.pdf (\([0-9]*\) pages.*/\1/p' "${HERE}/latex/pass2.log" | tail -1)
if [ -z "${PAGES}" ]; then
    echo "the compile did not report a page count; see book/latex/pass2.log" >&2
    exit 1
fi
if [ "${PAGES}" -lt 100 ]; then
    echo "implausible page count: ${PAGES}" >&2
    exit 1
fi
echo "  ${PAGES} pages"

if [ "${COPY}" -eq 1 ]; then
    cp "${HERE}/latex/main.pdf" "${REPO}/${OUT_NAME}"
    echo "  copied to ${OUT_NAME} ($(du -h "${REPO}/${OUT_NAME}" | cut -f1))"
fi
echo "book: OK"
