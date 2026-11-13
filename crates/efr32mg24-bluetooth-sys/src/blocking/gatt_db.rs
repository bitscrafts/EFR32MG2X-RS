//! A GATT database built at run time, from the stack's own BGAPI calls.
//!
//! The usual way to get a GATT database in this SDK is to generate `gatt_db.c` from the project's `.slcp` and compile
//! it into the application. This project does not do that: the database is built here with `sl_bt_gattdb_*` commands,
//! in Rust, so the service a firmware serves is visible in the same file as the code that serves it, and no generated C
//! enters the tree.
//!
//! Measured reason this module exists: `ble_gatt` connected to a real client and listed **zero characteristics**,
//! because it used a hand-written `CharHandle(0x000B)` that no attribute ever occupied. A connection succeeding says
//! nothing about what the device can serve -- only the client's attribute listing does.
//!
//! ## The sequence, and why the order matters
//!
//! 1. `new_session` — the database is edited in a transaction, not attribute by attribute.
//! 2. `add_service` — a primary service declaration with a 128-bit UUID.
//! 3. `add_uuid128_characteristic` — the value attribute, with the properties a client will see.
//! 4. `start_service` — the service becomes visible to clients. A service added but not started is absent from
//!    discovery, which looks exactly like a failed add.
//! 5. `commit` — the session's edits become the live database.
//!
//! ## Property bits
//!
//! From `sl_bt_api.h`, lower byte of the property word: read `0x02`, write without response `0x04`, write `0x08`,
//! notify `0x10`, indicate `0x20`, extended properties `0x80`.

use super::error::BleError;
use super::types::CharHandle;
use crate::ffi as bt;

/// The service type for a primary service, from `sl_bt_gattdb_service_type`.
const PRIMARY_SERVICE: u8 = 0x00;
/// `sl_bt_gattdb_user_managed_value`: the value lives in the stack's attribute, and this code writes it.
const USER_MANAGED_VALUE: u8 = 0x03;
/// `SL_BT_GATTDB_CHARACTERISTIC_READ`.
pub const PROP_READ: u16 = 0x02;
/// `SL_BT_GATTDB_CHARACTERISTIC_WRITE`.
pub const PROP_WRITE: u16 = 0x08;
/// `SL_BT_GATTDB_CHARACTERISTIC_NOTIFY`.
pub const PROP_NOTIFY: u16 = 0x10;

/// The handles of a service this code built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceHandles {
    /// The service declaration handle, as returned by `add_service`.
    pub service: u16,
    /// One characteristic value handle per characteristic, in the order they were added.
    ///
    /// These are the handles a client sees and the ones `notify` and `write_attribute_value` take. They are **not**
    /// stable across firings: they are assigned by the stack, and the only safe way to use one is to keep the value
    /// this call returned.
    pub characteristics: heapless::Vec<u16, 8>,
}

impl ServiceHandles {
    /// The handle of the characteristic at `index`, if it was added.
    pub fn characteristic(&self, index: usize) -> Option<CharHandle> {
        self.characteristics.get(index).copied().map(CharHandle)
    }
}

/// Add a primary service with a 128-bit UUID and the characteristics described by `characteristics`.
///
/// Each entry is `(uuid, properties)`, where `properties` is a bitmask of [`PROP_READ`], [`PROP_WRITE`] and
/// [`PROP_NOTIFY`].
///
/// The whole thing is one session: on any failure the session is not committed, so the database keeps the state it had
/// -- a half-added service is not a state a client can discover.
pub fn add_service(
    uuid: [u8; 16],
    characteristics: &[([u8; 16], u16)],
) -> Result<ServiceHandles, BleError> {
    let mut session: u16 = 0;
    let status = unsafe { bt::sl_bt_gattdb_new_session(&mut session) };
    if status != 0 {
        return Err(BleError::BgapiError(status as u16));
    }

    let mut service: u16 = 0;
    let status = unsafe {
        bt::sl_bt_gattdb_add_service(session, PRIMARY_SERVICE, 0, 16, uuid.as_ptr(), &mut service)
    };
    if status != 0 {
        return Err(BleError::BgapiError(status as u16));
    }

    let mut handles: heapless::Vec<u16, 8> = heapless::Vec::new();
    for (char_uuid, properties) in characteristics {
        // `uuid_128` is the 16 bytes by value, in the stack's little-endian order.
        let uuid_value: bt::uuid_128 = bt::uuid_128 { data: *char_uuid };
        let mut handle: u16 = 0;
        let status = unsafe {
            bt::sl_bt_gattdb_add_uuid128_characteristic(
                session,
                service,
                *properties,
                0, // no security requirement: an unpaired client may use it
                0, // no option flags
                uuid_value,
                USER_MANAGED_VALUE,
                0, // maxlen is ignored for a user-managed value
                0, // no initial value
                core::ptr::null(),
                &mut handle,
            )
        };
        if status != 0 {
            return Err(BleError::BgapiError(status as u16));
        }
        let _ = handles.push(handle);
    }

    let status = unsafe { bt::sl_bt_gattdb_start_service(session, service) };
    if status != 0 {
        return Err(BleError::BgapiError(status as u16));
    }

    let status = unsafe { bt::sl_bt_gattdb_commit(session) };
    if status != 0 {
        return Err(BleError::BgapiError(status as u16));
    }

    Ok(ServiceHandles {
        service,
        characteristics: handles,
    })
}
