//! Fast Start RCO. Registers of `FSRCO_NS`.
#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    ipversion: Ipversion,
}
impl RegisterBlock {
    #[doc = "0x00 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ipversion(&self) -> &Ipversion {
        &self.ipversion
    }
}
#[doc = "IPVERSION (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, FSRCO chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ipversion::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ipversion`] module"]
#[doc(alias = "IPVERSION")]
pub type Ipversion = crate::Reg<ipversion::IpversionSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, FSRCO chapter"]
pub mod ipversion;
