# display

Display drivers for the Expansion Base.

## What it is

Display drivers for the Expansion Base.

`ssd1306` is the 128x64 OLED that sits on the base board; the driver is
generic over the I2C transport so it runs on the bit-banged bus today and on
the on-chip I2C0 once that read path works.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `DEFAULT_ADDR` | const | Address the Expansion Base OLED answers on. |
| `Ssd1306` | struct | A 128x64 SSD1306 panel with a 1 KiB framebuffer. |
| `append` | fn | Append `src` to `dst` at `pos`, returning the new position. |
| `clear` | fn | Clear the framebuffer (does not touch the panel until [`flush`]). |
| `clear_display` | fn | Blank the panel and push the empty frame. |
| `flush` | fn | Push the whole framebuffer to the panel. |
| `format_fixed` | fn | Format a value in hundredths with `decimals` digits into `out`, returning the number of bytes written. |
| `free` | fn | Give the bus back. |
| `new` | fn | Probe, then initialise. |
| `text` | fn | Draw text at `column`, page `0..8`. |
| `with_address` | fn | Same as [`Self::new`] but for a panel strapped to another address. |

## Registers it touches

None directly: this module composes other drivers and reaches no `pac::` peripheral of its own.

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

Nothing in `docs/` records an unproven path for this module.

## Using it

No example imports this module yet, so its API has never been exercised from `examples/`.
