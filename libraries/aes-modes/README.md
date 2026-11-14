# aes-modes

AES-CMAC, AES-CTR and AES-ECB over a caller-supplied block cipher.

The block cipher is a trait parameter, so this crate has no opinion about hardware. AES-CMAC is checked against the
RFC 4493 test vectors in the unit tests.

```rust
use aes_modes::{cmac, BlockCipher};
```

`no_std`, no dependencies.
