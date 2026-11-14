# aes-ctr-radio

AES-CTR with the counter convention a Bluetooth stack expects.

Kept separate from `aes-modes` because the counter layout is a radio decision, not a cryptographic one: the stack builds
the counter block for you and a general-purpose CTR mode is the wrong shape for it.

`no_std`, no dependencies.
