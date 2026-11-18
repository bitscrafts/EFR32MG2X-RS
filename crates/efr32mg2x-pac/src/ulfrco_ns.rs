//! Ultra Low Frequency RC Oscillator. Registers of `ULFRCO_NS`.
#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    ipversion: Ipversion,
    _reserved1: [u8; 0x04],
    status: Status,
    _reserved2: [u8; 0x08],
    if_: If,
    ien: Ien,
}
impl RegisterBlock {
    #[doc = "0x00 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ipversion(&self) -> &Ipversion {
        &self.ipversion
    }
    #[doc = "0x08 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn status(&self) -> &Status {
        &self.status
    }
    #[doc = "0x14 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn if_(&self) -> &If {
        &self.if_
    }
    #[doc = "0x18 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ien(&self) -> &Ien {
        &self.ien
    }
}
#[doc = "IPVERSION (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ULFRCO chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ipversion::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ipversion`] module"]
#[doc(alias = "IPVERSION")]
pub type Ipversion = crate::Reg<ipversion::IpversionSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ULFRCO chapter"]
pub mod ipversion;
#[doc = "STATUS (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ULFRCO chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status`] module"]
#[doc(alias = "STATUS")]
pub type Status = crate::Reg<status::StatusSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ULFRCO chapter"]
pub mod status;
#[doc = "IF (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ULFRCO chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`if_::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`if_::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@if_`] module"]
#[doc(alias = "IF")]
pub type If = crate::Reg<if_::IfSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ULFRCO chapter"]
pub mod if_;
#[doc = "IEN (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ULFRCO chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ien`] module"]
#[doc(alias = "IEN")]
pub type Ien = crate::Reg<ien::IenSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ULFRCO chapter"]
pub mod ien;
