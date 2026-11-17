#!/usr/bin/env bash
# audit-blobs.sh — what the Bluetooth blobs import, export and call.
#
# Exists so a review (or a person) can ask the binaries a question instead of guessing from the SDK headers.
# Every question below was an actual dead end in this project until it was asked this way.
#
#   bash scripts/audit-blobs.sh imports              every undefined symbol across the three archives
#   bash scripts/audit-blobs.sh psa                  only the PSA/mbedtls imports, which is the pairing path
#   bash scripts/audit-blobs.sh callers <symbol>     which object file references a given symbol
#   bash scripts/audit-blobs.sh disasm <symbol>      the disassembly of that symbol, if the archive defines it
#   bash scripts/audit-blobs.sh strings <pattern>    string literals matching a pattern
#   bash scripts/audit-blobs.sh docs                 where the EFR32 documentation lives and how to search it
set -u

BLOBS="$(dirname "$0")/../crates/efr32mg24-bluetooth-sys/blobs"
RM_PDF="/projects/_tmp/efr32xg24-rm.pdf"
DS_PDF="/projects/_tmp/efr32mg24-datasheet.pdf"

case "${1:-}" in
  imports)
    nm -u "$BLOBS"/*/*.a 2>/dev/null | awk '{print $2}' | grep -E '^[A-Za-z_]' | sort -u
    ;;
  psa)
    nm -u "$BLOBS"/*/*.a 2>/dev/null | awk '{print $2}' \
      | grep -oE '\b(psa|mbedtls)_[a-z_0-9]+' | sort -u
    ;;
  callers)
    sym="${2:?usage: audit-blobs.sh callers <symbol>}"
    for a in "$BLOBS"/*/*.a; do
      nm -A "$a" 2>/dev/null | grep -E " U $sym\$" | sed 's/:.*U /  -> /' | head -5
    done
    ;;
  disasm)
    sym="${2:?usage: audit-blobs.sh disasm <symbol>}"
    for a in "$BLOBS"/*/*.a; do
      if nm "$a" 2>/dev/null | grep -qE " T $sym\$"; then
        echo "== $sym in $a"
        objdump -d --disassemble="$sym" "$a" 2>/dev/null | head -80
        exit 0
      fi
    done
    echo "not defined in any blob: $sym (it is imported, or it lives in the vendor sources)"
    ;;
  strings)
    pat="${2:?usage: audit-blobs.sh strings <pattern>}"
    strings "$BLOBS"/*/*.a 2>/dev/null | grep -iE "$pat" | sort -u | head -30
    ;;
  docs)
    cat <<EOF
Reference manual   $RM_PDF   (1152 pages; search it with:)
  cd /projects/_tmp && uv run --with pymupdf python -c "
import pymupdf,sys,re
d=pymupdf.open('efr32xg24-rm.pdf')
for i,p in enumerate(d):
    t=p.get_text()
    if re.search(sys.argv[1], t, re.I): print(i+1, ' '.join(t.split())[:200])
" 'SM |security manager|PSA'

Datasheet          $DS_PDF
How to fetch them  scripts/fetch-reference-manual.sh, scripts/fetch-datasheet.sh
                   (both need the full Chrome header set: a plain curl or wget gets HTTP 403 from Silabs)
EOF
    ;;
  *)
    sed -n '2,14p' "$0"
    ;;
esac
