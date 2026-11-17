## Sources and tools you are expected to use

Documentation is in the tree and searching it is part of the job, not a nicety: the IADC timer question was only
answered because the reference manual was there and the reviewer read it.

```
manual      /projects/_tmp/efr32xg24-rm.pdf          1152 pages, script: scripts/fetch-reference-manual.sh
datasheet   /projects/_tmp/efr32mg24-datasheet.pdf   script: scripts/fetch-datasheet.sh
            both need the full Chrome header set — a plain curl or wget gets HTTP 403 from Silabs
blobs       bash scripts/audit-blobs.sh imports | psа | callers <symbol> | disasm <symbol> | strings <pat>
            nm and objdump answer "who calls this" and "does this symbol exist", definitively and cheaply
vendor C    crates/efr32mg24-emlib-sys/vendor/emlib/{src,inc}    the SDK's own implementation, when present
docs help   bash scripts/audit-blobs.sh docs          prints the recipe for searching the manual by regex
```

Ask the binaries before the headers. `bash scripts/audit-blobs.sh callers psa_generate_key` already shows that the
P-256 key generation lives in the **host** blob — which is the fact that makes our PSA shim part of the pairing
path at all, and it took one command rather than an argument.

When a brief's question is "why does X happen", name the artefact that would settle it (a page, a symbol, a
register, a measurement on the board) and say who can obtain it. A reviewer with tools answers; a reviewer
without them speculates fluently.
