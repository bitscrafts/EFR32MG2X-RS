# bt-heap

A first-fit heap for the Silicon Labs Bluetooth stack.

`no_std`, no dependencies, and built to be inspected rather than trusted: every block carries a canary, the last
allocations are kept in a ring of (size, offset) pairs, a double free is counted instead of corrupting the list, and
`audit()` reports how many blocks are damaged and which one the walk stopped at.

The 256 bytes of slack per block are not a precaution, they are a measurement: a vendor path in the Bluetooth stack
writes about 136 bytes into a 64-byte allocation, and without the slack that write lands on the next block's header and
takes the rest of the arena out of reach.

Tested on the host:

```bash
cargo test -p bt-heap --target x86_64-unknown-linux-gnu
```
