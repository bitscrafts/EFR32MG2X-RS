//! The Radio Configurator's BLE 1 Mbps Viterbi 39 MHz configuration, as Rust data.
//!
//! **Generated** by `/projects/_tmp/efr32mg24-rs/emit-ble-config.py` from the Gecko SDK's own
//! `sl_rail_ble_config_39MHz.c` (Zlib, Silicon Laboratories). Regenerate rather than edit.
//!
//! Why this file exists: RAIL refuses to configure a radio whose crystal frequency it does not know, asserting
//! `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` (code 66), and the default `RAILCb_AssertFailed` implementation is an
//! infinite loop -- so a Bluetooth stack that starts without this table looks like it hangs. The frequency lives in
//! `RAIL_ChannelConfig_t.xtalFrequencyHz`, and it arrives here as data rather than as compiled C because including
//! `rail_types.h` in our own translation unit duplicates symbols the RAIL blob already defines.
//!
//! The tables are `static mut` and filled on first use because ten of their slots hold **addresses as integers**
//! (`(uint32_t) &phyInfo_0`, `(uint32_t) irCalConfig_0`, ...), and Rust will not evaluate a pointer-to-integer cast
//! in a constant.

#![allow(non_upper_case_globals)]

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::{
    RAIL_ChannelConfigEntryAttr_t, RAIL_ChannelConfigEntry_t, RAIL_ChannelConfig_t,
    RAIL_RadioConfig_t,
};

static MODEM_CONFIG_BASE: [u32; 180] = [
    81932, 98561, 147488, 0, 1, 213040, 0, 0, 0, 147520, 0, 16384, 82016, 257, 82088, 7, 278716, 0,
    0, 0, 0, 213260, 19711, 19967, 19967, 268550176, 522240, 805421088, 19399413, 639012, 4864,
    62095596, 1363178819, 4160753568, 16384, 502440, 0, 0, 0, 639088, 4282, 2098176, 8394756,
    18889739, 34634776, 115407919, 127, 0, 0, 377000, 359812029, 85500689, 1985229328, 2712, 0,
    311500, 1, 0, 655361, 2621441, 16842760, 1800, 16842776, 11184810, 16842784, 14311424,
    16924764, 50331648, 536870912, 17055864, 1903575337, 0, 786572, 0, 17055940, 0, 0, 0, 0,
    16859360, 512, 16924944, 335411, 0, 17187104, 0, 126027007, 986190728, 208013055, 126027007,
    67048328, 16859480, 0, 16859492, 268, 17121668, 257, 0, 0, 0, 0, 16925092, 0, 0, 16925104, 0,
    3225326168, 16990652, 0, 3932160, 430185, 16990764, 1073748064, 0, 0, 16859708, 0, 16925252,
    20, 0, 16925312, 0, 129, 17121944, 33554495, 65535, 65535, 1023, 65535, 16859828, 0, 17449776,
    18874432, 160, 16797704, 522133279, 455024415, 286463256, 202182160, 797425989, 0, 0, 16875536,
    3, 16875580, 1, 16875676, 0, 16941232, 33555200, 16777271, 33620124, 67111936, 33751256,
    2856321029, 392, 192, 33620208, 299, 302055696, 1048320, 838926608, 822083586, 302055760,
    114688, 838926672, 10617025, 33620340, 203157865, 302055800, 1835008, 838926712, 3487564816,
    302055808, 1913, 838926720, 2, 33620360, 80, 4294967295,
];

static mut MODEM_CONFIG_0_37: [u32; 120] = [
    50417660, 0, 81936, 16641, 81980, 0, 81992, 50333600, 82000, 0, 82012, 0, 82184, 19711, 507988,
    3150161, 3859349518, 45, 117835012, 8, 177873, 13668, 17252416, 1364197376, 295698432, 16,
    2248146944, 8503327, 536870912, 0, 17121380, 1073741824, 16254137, 2112, 2, 262290, 17776776,
    3867575, 3758096384, 0, 218432000, 1396846364, 1617455458, 523124819, 134744593, 101124104,
    50529285, 16908802, 16843009, 0, 0, 0, 16859420, 2360434688, 17252664, 4027195580, 5399286,
    1083726597, 2491575950, 1380481052, 4209545, 2147484608, 16859484, 56831, 17252712, 16405480,
    1073741824, 0, 0, 0, 0, 0, 16990672, 2852126720, 208, 1179652, 18039264, 0, 167816191, 1044828,
    147923, 11660693, 265845529, 79235090, 527244266, 180533193, 3874827, 1603, 15950425, 9862770,
    1654183, 1005658832, 2684354560, 0, 56, 0, 16859724, 67305480, 16859752, 1, 16875576, 1057986,
    16941216, 227440, 208, 285311144, 496, 822182056, 30097925, 285311148, 496, 822182060,
    16596485, 302055660, 4064, 838926572, 1364271116, 33620356, 4096, 4294967295,
];

static mut PHY_INFO: [u32; 20] = [
    16, 9586980, 0, 0, 0, 0, 0, 16000000, 1000000, 16253185, 55583938, 0, 0, 0, 0, 0, 999979, 0, 0,
    0,
];

static IR_CAL_CONFIG: [u8; 26] = [
    25, 63, 1, 6, 4, 16, 1, 0, 0, 1, 1, 6, 0, 16, 39, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0,
];

static TIMING_CONFIG: [i32; 3] = [50000, 11250, 750];

static HFXO_RETIMING: [u8; 86] = [
    2, 0, 0, 0, 192, 23, 83, 2, 6, 20, 0, 0, 128, 47, 166, 4, 5, 56, 0, 0, 160, 8, 0, 0, 0, 0, 88,
    9, 1, 4, 7, 6, 16, 10, 1, 4, 7, 7, 200, 10, 0, 4, 8, 7, 128, 11, 0, 4, 8, 8, 56, 12, 0, 4, 9,
    8, 97, 8, 0, 0, 0, 0, 138, 8, 0, 0, 0, 0, 199, 9, 1, 4, 4, 3, 44, 11, 1, 4, 4, 4, 146, 12, 1,
    4, 5, 4,
];

static STACK_INFO: [u8; 2] = [3, 0];

static mut ATTR: MaybeUninit<RAIL_ChannelConfigEntryAttr_t> = MaybeUninit::uninit();
static mut ENTRIES: MaybeUninit<[RAIL_ChannelConfigEntry_t; 3]> = MaybeUninit::uninit();
static mut CONFIG: MaybeUninit<RAIL_ChannelConfig_t> = MaybeUninit::uninit();
static READY: AtomicBool = AtomicBool::new(false);

// The pointer slots, written once. Each list is the set of indices in that array whose value is an address.
unsafe fn patch_modem_config_0_37() {
    MODEM_CONFIG_0_37[1] = core::ptr::addr_of!(PHY_INFO) as u32;
}

unsafe fn patch_phy_info() {
    PHY_INFO[2] = 0;
    PHY_INFO[3] = core::ptr::addr_of!(IR_CAL_CONFIG) as u32;
    PHY_INFO[4] = core::ptr::addr_of!(TIMING_CONFIG) as u32;
    PHY_INFO[11] = 0;
    PHY_INFO[12] = core::ptr::addr_of!(HFXO_RETIMING) as u32;
    PHY_INFO[13] = 0;
    PHY_INFO[17] = 0;
    PHY_INFO[18] = 0;
    PHY_INFO[19] = 0;
}

/// The channel configuration, filled on first call and valid for the rest of the program.
pub fn config() -> &'static RAIL_ChannelConfig_t {
    if !READY.swap(true, Ordering::AcqRel) {
        // SAFETY: guarded by `READY`; the storage is `static`, so the reference handed out below outlives any use.
        unsafe {
            patch_phy_info();
            patch_modem_config_0_37();

            // The attribute block is all-0xFF in the source; filled as bytes so this file does not depend on the
            // shape of `RAIL_RxIrCalValues_t`.
            core::ptr::write_bytes(
                core::ptr::addr_of_mut!(ATTR) as *mut u8,
                0xFF,
                core::mem::size_of::<RAIL_ChannelConfigEntryAttr_t>(),
            );

            let entries = core::ptr::addr_of_mut!(ENTRIES) as *mut RAIL_ChannelConfigEntry_t;
            let table: [RAIL_ChannelConfigEntry_t; 3] = [
                RAIL_ChannelConfigEntry_t {
                    phyConfigDeltaAdd: core::ptr::addr_of!(MODEM_CONFIG_0_37) as RAIL_RadioConfig_t,
                    baseFrequency: 2402000000,
                    channelSpacing: 2000000,
                    physicalChannelOffset: 0u16,
                    channelNumberStart: 0u16,
                    channelNumberEnd: 37u16,
                    maxPower: i16::MAX,
                    attr: core::ptr::addr_of_mut!(ATTR) as *mut RAIL_ChannelConfigEntryAttr_t,
                    entryType: 0,
                    reserved: [0; 3],
                    stackInfo: core::ptr::addr_of!(STACK_INFO) as *const u8,
                    alternatePhy: core::ptr::null_mut(),
                },
                RAIL_ChannelConfigEntry_t {
                    phyConfigDeltaAdd: core::ptr::addr_of!(MODEM_CONFIG_0_37) as RAIL_RadioConfig_t,
                    baseFrequency: 2402000000,
                    channelSpacing: 2000000,
                    physicalChannelOffset: 0u16,
                    channelNumberStart: 38u16,
                    channelNumberEnd: 38u16,
                    maxPower: 183i16,
                    attr: core::ptr::addr_of_mut!(ATTR) as *mut RAIL_ChannelConfigEntryAttr_t,
                    entryType: 0,
                    reserved: [0; 3],
                    stackInfo: core::ptr::addr_of!(STACK_INFO) as *const u8,
                    alternatePhy: core::ptr::null_mut(),
                },
                RAIL_ChannelConfigEntry_t {
                    phyConfigDeltaAdd: core::ptr::addr_of!(MODEM_CONFIG_0_37) as RAIL_RadioConfig_t,
                    baseFrequency: 2402000000,
                    channelSpacing: 2000000,
                    physicalChannelOffset: 0u16,
                    channelNumberStart: 39u16,
                    channelNumberEnd: 39u16,
                    maxPower: 140i16,
                    attr: core::ptr::addr_of_mut!(ATTR) as *mut RAIL_ChannelConfigEntryAttr_t,
                    entryType: 0,
                    reserved: [0; 3],
                    stackInfo: core::ptr::addr_of!(STACK_INFO) as *const u8,
                    alternatePhy: core::ptr::null_mut(),
                },
            ];
            for (i, e) in table.into_iter().enumerate() {
                core::ptr::write(entries.add(i), e);
            }

            core::ptr::write(
                core::ptr::addr_of_mut!(CONFIG) as *mut RAIL_ChannelConfig_t,
                RAIL_ChannelConfig_t {
                    phyConfigBase: core::ptr::addr_of!(MODEM_CONFIG_BASE) as RAIL_RadioConfig_t,
                    phyConfigDeltaSubtract: core::ptr::null(),
                    configs: core::ptr::addr_of!(ENTRIES) as *const RAIL_ChannelConfigEntry_t,
                    length: 3,
                    signature: 0,
                    xtalFrequencyHz: 39000000,
                },
            );
        }
    }
    // SAFETY: written exactly once above, before any reference is handed out.
    unsafe { &*core::ptr::addr_of!(CONFIG).cast::<RAIL_ChannelConfig_t>() }
}
