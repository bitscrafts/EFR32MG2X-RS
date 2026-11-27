//! `RADIOAES`, the on-chip AES engine, driven in scatter-gather mode: one block in, one block out.
//!
//! # Why this driver looks like a DMA driver
//!
//! The block has no `KEY`, `DATA` or `MODE` register. It has a fetcher and a pusher that walk descriptor lists in RAM
//! (`{address, next, length, tag}`, four words each), and the parameter space of the crypto core is filled by *blocks*
//! in the fetcher chain whose tag says where they land: the key at parameter offset 8, the mode word at offset 0. The
//! engine then reads the payload through the same chain and writes the result through the pusher.
//!
//! That interface is the one the vendor's own driver uses for the BLE stack (`sli_protocol_crypto_radioaes.c`, vendored
//! in `crates/efr32mg24-bluetooth-sys/vendor/support`); the tag arithmetic is re-exported from `cores::aes` so a reader
//! can check the words this module writes without reading 800 lines of vendor C.
//!
//! # What was measured, and what is only claimed
//!
//! Measured on the XIAO MG24 Sense (2026-11-27, `docs/04-reference/AES-KAT-2026-11-27.md`): the two published vectors
//! (FIPS-197 C.1 and NIST SP 800-38A F.1.1 block 1) come back correct from this driver, the decrypt bit returns the
//! plaintext, and the busy bits are clear when the call returns. Everything else -- CTR, CBC, CCM, keys other than 128
//! bits, and the interrupt path -- is not exposed here, because no instrument for it exists in this crate and a driver
//! without one is a claim.
//!
//! # The radio is not touched
//!
//! `RADIOAES` sits in chapter 5 of the reference manual and shares the radio's address neighbourhood, but it is its own
//! block in the **EM1** power domain with its own bus gate (`CMU_CLKEN0.RADIOAES`, bit 2), and it answers with
//! `CMU_RADIOCLKCTRL` left exactly as it was found. Nothing in this module writes a radio register, and the example that
//! uses it never enables the transceiver: the measurement is a cipher, not a transmission.

use efr32mg2x_pac as pac;

pub use crate::cores::aes::{
    answered, config_tag, data_tag, differences, ecb_control, is_busy, length_word, push_tag,
    KnownAnswer, CFG_OFFSET_KEY, CFG_OFFSET_MODE, FIPS197, SP800_38A_ECB_BLOCK1,
};

/// How many times the status register is read before an operation is declared stuck.
const STATUS_POLLS: u32 = 4_000_000;

/// Why an `RADIOAES` operation was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The crypto core did not identify itself: `BA411E_HW_CFG_1` read as all-zero or all-ones, which is a block that is
    /// not clocked or not there, not a config word.
    NotPresent {
        /// The word read from `BA411E_HW_CFG_1`.
        config: u32,
    },
    /// The descriptors were started and the fetcher or the pusher never went idle again.
    NeverDone {
        /// The last `STATUS` word read.
        status: u32,
        /// How many polls were made.
        polls: u32,
    },
}

/// One scatter-gather descriptor, as the engine fetches it: four words, sixteen bytes, 16-byte aligned.
///
/// The alignment is the engine's requirement, not a convenience: the low bits of a `next` pointer carry the "stop"
/// marker, so a descriptor that is not word-aligned cannot be named by one.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct Descriptor {
    /// Where the block is in RAM, or in the crypto core's parameter space after the tag says `config`.
    address: u32,
    /// The next descriptor in the chain, or the low-bit "stop" marker.
    next: u32,
    /// Length in bytes, low 28 bits.
    length: u32,
    /// Engine selector, what the block is, and where a configuration block lands.
    tag: u32,
}

/// Sixteen bytes of RAM the engine can fetch from or push to, word-aligned.
///
/// A caller's `&[u8; 16]` has byte alignment; the block's bus master reads words, so the copies this driver makes are
/// aligned on purpose rather than left to the stack's layout.
#[repr(C, align(4))]
struct Block([u8; 16]);

/// Fill one descriptor in place.
///
/// The writes are volatile because nothing in Rust knows the engine reads this memory: a plain store into a struct no
/// other code touches is exactly what an optimiser is entitled to delete.
fn write_descriptor(slot: &mut Descriptor, address: u32, next: u32, length: u32, tag: u32) {
    let p = slot as *mut Descriptor;
    // SAFETY: `p` is a valid, aligned, uniquely borrowed descriptor; the four writes cover the whole struct and none of
    // them is a read-modify-write, so there is no previous value to preserve.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*p).address), address);
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*p).next), next);
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*p).length), length);
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*p).tag), tag);
    }
}

/// Copy sixteen bytes into engine-visible RAM with volatile stores.
///
/// Like the descriptors: this memory has a reader outside the Rust machine, so the stores must not be deferred or
/// elided while the program looks as though nobody reads them.
fn fill_block(slot: &mut Block, bytes: &[u8; 16]) {
    let p = slot.0.as_mut_ptr();
    for (i, byte) in bytes.iter().enumerate() {
        // SAFETY: `slot` owns sixteen bytes and `i < 16`, so every store is inside the array.
        unsafe { core::ptr::write_volatile(p.add(i), *byte) };
    }
}

/// A descriptor's address as the engine wants it: a plain 32-bit bus address.
fn address_of<T>(value: &mut T) -> u32 {
    (value as *mut T) as usize as u32
}

/// The AES engine, owned by one caller through the board's claim table.
pub struct Aes {
    regs: pac::RadioaesS,
}

impl Aes {
    /// Take the block, and refuse it if it does not identify itself.
    ///
    /// # Why there is no reset here
    ///
    /// The obvious warm-up -- write `CTRL.SWRESET`, wait out `STATUS.SOFTRSTBSY` -- never returns on this part. Measured
    /// (2026-11-27, first image of this item): at reset `STATUS` reads `0x0000_0040` with `CTRL` reading `0x0000_0010`,
    /// i.e. the block comes out of reset with `SWRESET` already set; writing `SWRESET` again leaves both words unchanged,
    /// and clearing `CTRL` to zero leaves `SOFTRSTBSY` set as well. 200 000 polls later it was still set, so the flag is
    /// not a completion handshake and waiting on it is a boot that reports nothing. The vendor driver for the BLE stack
    /// never writes `SWRESET` either: its first write is `CTRL = FETCHERSCATTERGATHER | PUSHERSCATTERGATHER`, which is
    /// what [`Aes::run`] does, and that write drops the reset bit on the way past.
    ///
    /// What this function does check is that the block answered at all: `BA411E_HW_CFG_1` is a constant the core
    /// publishes about itself (measured `0x0503_0127` on this board), and an all-zero or all-ones read is a clock that is
    /// off or a block that is not there, not a configuration.
    pub fn new(regs: pac::RadioaesS) -> Result<Self, Error> {
        let aes = Aes { regs };
        let config = aes.regs.ba411e_hw_cfg_1().read().bits();
        if config == 0 || config == u32::MAX {
            return Err(Error::NotPresent { config });
        }
        Ok(aes)
    }

    /// Encrypt one 16-byte block into `out`, under a 128-bit key.
    pub fn encrypt_block(
        &mut self,
        key: &[u8; 16],
        input: &[u8; 16],
        out: &mut [u8; 16],
    ) -> Result<(), Error> {
        self.run(key, input, out, false)
    }

    /// Decrypt one 16-byte block into `out`, under a 128-bit key.
    pub fn decrypt_block(
        &mut self,
        key: &[u8; 16],
        input: &[u8; 16],
        out: &mut [u8; 16],
    ) -> Result<(), Error> {
        self.run(key, input, out, true)
    }

    /// Program the four descriptors, start both halves and wait for the engine to go idle.
    fn run(
        &mut self,
        key: &[u8; 16],
        input: &[u8; 16],
        out: &mut [u8; 16],
        decrypt: bool,
    ) -> Result<(), Error> {
        // The engine reads these through its own bus master, so each of them has to be ordinary, word-aligned RAM that
        // outlives the call -- copies on this stack frame, not the caller's arguments (which may be in flash, and a
        // fetcher pointed at flash would fetch the constant instead of the data).
        let mut mode: u32 = 0;
        let mut key_copy = Block([0u8; 16]);
        let mut source = Block([0u8; 16]);
        fill_block(&mut key_copy, key);
        fill_block(&mut source, input);
        // SAFETY: `&mut mode` is a valid, aligned word this frame owns.
        unsafe { core::ptr::write_volatile(&mut mode, ecb_control(decrypt)) };

        let mut payload = Descriptor {
            address: 0,
            next: 0,
            length: 0,
            tag: 0,
        };
        let mut mode_block = payload;
        let mut key_block = payload;
        let mut pusher = payload;

        let source_addr = address_of(&mut source);
        let output_addr = (out.as_mut_ptr() as usize) as u32;
        let mode_addr = address_of(&mut mode);
        let key_addr = address_of(&mut key_copy);

        // The addresses have to be taken before the chain is written, because each descriptor names the next one.
        let payload_addr = address_of(&mut payload);
        let mode_block_addr = address_of(&mut mode_block);
        let key_block_addr = address_of(&mut key_block);
        let pusher_addr = address_of(&mut pusher);

        // The chain, in the order the engine walks it: key -> mode -> payload; and the pusher holds the result.
        write_descriptor(
            &mut key_block,
            key_addr,
            mode_block_addr,
            length_word(16),
            config_tag(CFG_OFFSET_KEY),
        );
        write_descriptor(
            &mut mode_block,
            mode_addr,
            payload_addr,
            length_word(4),
            config_tag(CFG_OFFSET_MODE),
        );
        write_descriptor(
            &mut payload,
            source_addr,
            crate::cores::aes::NEXT_STOP,
            length_word(16),
            data_tag(true),
        );
        write_descriptor(
            &mut pusher,
            output_addr,
            crate::cores::aes::NEXT_STOP,
            length_word(16),
            push_tag(true),
        );

        // The descriptors are in memory and the engine is about to read them; nothing may be reordered past this point.
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        // SAFETY: the writes are whole-word register writes whose bits are the fields the vendor's scatter-gather
        // sequence sets (`CTRL` = fetcher and pusher both in scatter-gather mode); the addresses written to `FETCHADDR`
        // and `PUSHADDR` name `Descriptor` values and RAM words that live on this frame for the duration of the wait,
        // which is where the engine reads them from.
        unsafe {
            self.regs
                .ctrl()
                .write(|w| w.bits(crate::cores::aes::CTRL_SCATTER_GATHER));
            self.regs.fetchaddr().write(|w| w.bits(key_block_addr));
            self.regs.pushaddr().write(|w| w.bits(pusher_addr));
        }
        self.regs
            .cmd()
            .write(|w| w.startfetcher().set_bit().startpusher().set_bit());

        let mut polls = 0u32;
        while is_busy(self.status()) {
            polls += 1;
            if polls > STATUS_POLLS {
                return Err(Error::NeverDone {
                    status: self.status(),
                    polls,
                });
            }
        }
        // The descriptors are still alive here and the engine has stopped reading them -- the busy bits are clear -- so
        // they can go out of scope at the end of this frame without pulling memory out from under a running fetch.
        Ok(())
    }

    /// `STATUS`: the two busy bits, the FIFO level and the waiting flag, as one word.
    #[must_use]
    pub fn status(&self) -> u32 {
        self.regs.status().read().bits()
    }

    /// `IF`: fetcher and pusher end-of-block, stopped and error flags, as one word.
    #[must_use]
    pub fn flags(&self) -> u32 {
        self.regs.if_().read().bits()
    }

    /// Clear every flag in `IF`.
    pub fn clear_flags(&mut self) {
        // SAFETY: `IF_CLR`'s six defined bits are 0..=5; 0x3f sets exactly those and no reserved bit, so a write clears
        // every flag the block has without touching a field nobody defined.
        self.regs.if_clr().write(|w| unsafe { w.bits(0x3f) });
    }

    /// `BA411E_HW_CFG_1`: the identity of the crypto core, read from the block itself rather than assumed.
    #[must_use]
    pub fn core_config(&self) -> u32 {
        self.regs.ba411e_hw_cfg_1().read().bits()
    }
}
