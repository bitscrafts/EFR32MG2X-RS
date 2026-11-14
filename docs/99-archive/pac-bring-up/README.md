# Bring-up records, kept out of the published crate

These are the working notes from getting the first program to run on this board, between December 2025 and January
2026. They are kept because they hold measurements nobody wants to repeat — the CMU unlock key, the non-secure
hand-off, the bootloader's ApplicationProperties, the DOUTTGL that does not exist on this part.

They are **not** shipped in `efr32mg2x-pac` any more, because several of them say things like "blink timing issue
remains", which was true in January 2026 and is false now: `pac_timer_blink` was measured at 1.02 Hz against TIMER0 and
`pac_led_blink` was watched blinking on the board.

Read them as a diary, not as documentation. The current state lives in:

- `crates/efr32mg24-pac/docs/crate-doc.md` — what the crate is, and what the SVD does not tell you
- `crates/efr32mg24-pac/docs/examples.md` — every example, generated from the files
- `docs/03-development/STATUS.md` — the workspace's authoritative status
