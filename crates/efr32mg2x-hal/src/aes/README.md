# aes

`RADIOAES`, the on-chip AES engine, driven in scatter-gather mode: one block in, one block out.

## What it is

`RADIOAES`, the on-chip AES engine, driven in scatter-gather mode: one block in, one block out.

# Why this driver looks like a DMA driver

The block has no `KEY`, `DATA` or `MODE` register. It has a fetcher and a pusher that walk descriptor lists in RAM
(`{address, next, length, tag}`, four words each), and the parameter space of the crypto core is filled by *blocks*
in the fetcher chain whose tag says where they land: the key at parameter offset 8, the mode word at offset 0. The
engine then reads the payload through the same chain and writes the result through the pusher.

That interface is the one the vendor's own driver uses for the BLE stack (sli_protocol_crypto_radioaes.c, vendored
in `crates/efr32mg24-bluetooth-sys/vendor/support`); the tag arithmetic is re-exported from `cores::aes` so a reader
can check the words this module writes without reading 800 lines of vendor C.

# What was measured, and what is only claimed

Measured on the XIAO MG24 Sense (2026-11-27, `../../../../docs/../../../../docs/04-reference/AES-KAT-2026-11-27.md`): the two published vectors
(FIPS-197 C.1 and NIST SP 800-38A F.1.1 block 1) come back correct from this driver, the decrypt bit returns the
plaintext, and the busy bits are clear when the call returns. Everything else -- CTR, CBC, CCM, keys other than 128
bits, and the interrupt path -- is not exposed here, because no instrument for it exists in this crate and a driver
without one is a claim.

# The radio is not touched

`RADIOAES` sits in chapter 5 of the reference manual and shares the radio's address neighbourhood, but it is its own
block in the **EM1** power domain with its own bus gate (`CMU_CLKEN0.RADIOAES`, bit 2), and it answers with
`CMU_RADIOCLKCTRL` left exactly as it was found. Nothing in this module writes a radio register, and the example that
uses it never enables the transceiver: the measurement is a cipher, not a transmission.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Aes` | struct | The AES engine, owned by one caller through the board's claim table. |
| `Error` | enum | — |
| `clear_flags` | fn | Clear every flag in `IF`. |
| `core_config` | fn | — |
| `decrypt_block` | fn | Decrypt one 16-byte block into `out`, under a 128-bit key. |
| `encrypt_block` | fn | Encrypt one 16-byte block into `out`, under a 128-bit key. |
| `flags` | fn | — |
| `new` | fn | Take the block, and refuse it if it does not identify itself. |
| `status` | fn | — |

## Registers it touches

`RadioaesS`

## What was measured

```
    aes: ok=1 n=4 ct=69c4e0d86a7b0430d8cdb78070b4c55a c2=3ad77bb40d7a3660a89ecaf32466ef97
```

Controls and the full story: [AES-KAT-2026-11-27.md](../../../../docs/04-reference/AES-KAT-2026-11-27.md)

## What is not proven

- `libraries/` | `no_std` logic with no hardware: `bt-heap`, `aes-modes`, `aes-ctr-radio`, `motion-detection`, `record-core` | host-tested, never published |. (STATUS.md (`../../../../docs/03-development/STATUS`))

## Using it

See `examples/hal_aes_kat.rs`
