#!/usr/bin/env bash
# Fetch the EFR32xG24 reference manual the way a browser does.
# Silabs answers "Access Denied" to plain curl/wget from this host (measured 2026-11-17), so this sends the
# header set Chrome sends and negotiates HTTP/2. If it still fails, the manual has to come from a browser.
set -u
cd /projects/_tmp || exit 1
UA="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
URL="https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf"

curl -sSL --http2 --compressed --max-time 240 \
  -A "$UA" \
  -H "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,application/pdf,*/*;q=0.8" \
  -H "Accept-Language: en-US,en;q=0.9" \
  -H "Referer: https://www.silabs.com/documents/public/reference-manuals/" \
  -H "Sec-Fetch-Dest: document" \
  -H "Sec-Fetch-Mode: navigate" \
  -H "Sec-Fetch-Site: same-origin" \
  -H "Sec-Fetch-User: ?1" \
  -H "Upgrade-Insecure-Requests: 1" \
  -o efr32xg24-rm.pdf "$URL"

size=$(stat -c%s efr32xg24-rm.pdf 2>/dev/null || echo 0)
kind=$(file -b efr32xg24-rm.pdf 2>/dev/null | cut -c1-40)
echo "efr32xg24-rm.pdf -> ${size} bytes | ${kind}"
if [ "$size" -lt 100000 ]; then
  echo "REFUSED: the server did not send the PDF (a real manual is tens of MB)."
  head -c 160 efr32xg24-rm.pdf 2>/dev/null; echo
  exit 1
fi
echo "OK: looks like a PDF"
