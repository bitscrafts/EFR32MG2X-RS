//! Digital Phased Locked Loop. Registers of `DPLL0_S`.
#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    ipversion: Ipversion,
    en: En,
    cfg: Cfg,
    cfg1: Cfg1,
    if_: If,
    ien: Ien,
    status: Status,
    _reserved7: [u8; 0x08],
    lock: Lock,
}
impl RegisterBlock {
    #[doc = "0x00 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ipversion(&self) -> &Ipversion {
        &self.ipversion
    }
    #[doc = "0x04 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn en(&self) -> &En {
        &self.en
    }
    #[doc = "0x08 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn cfg(&self) -> &Cfg {
        &self.cfg
    }
    #[doc = "0x0c - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn cfg1(&self) -> &Cfg1 {
        &self.cfg1
    }
    #[doc = "0x10 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn if_(&self) -> &If {
        &self.if_
    }
    #[doc = "0x14 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ien(&self) -> &Ien {
        &self.ien
    }
    #[doc = "0x18 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn status(&self) -> &Status {
        &self.status
    }
    #[doc = "0x24 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn lock(&self) -> &Lock {
        &self.lock
    }
}
#[doc = "IPVERSION (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ipversion::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ipversion`] module"]
#[doc(alias = "IPVERSION")]
pub type Ipversion = crate::Reg<ipversion::IpversionSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter"]
pub mod ipversion;
#[doc = "EN (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`en::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`en::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@en`] module"]
#[doc(alias = "EN")]
pub type En = crate::Reg<en::EnSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter"]
pub mod en;
#[doc = "CFG (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cfg`] module"]
#[doc(alias = "CFG")]
pub type Cfg = crate::Reg<cfg::CfgSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter"]
pub mod cfg;
#[doc = "CFG1 (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cfg1`] module"]
#[doc(alias = "CFG1")]
pub type Cfg1 = crate::Reg<cfg1::Cfg1Spec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter"]
pub mod cfg1;
#[doc = "IF (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`if_::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`if_::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@if_`] module"]
#[doc(alias = "IF")]
pub type If = crate::Reg<if_::IfSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter"]
pub mod if_;
#[doc = "IEN (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ien`] module"]
#[doc(alias = "IEN")]
pub type Ien = crate::Reg<ien::IenSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter"]
pub mod ien;
#[doc = "STATUS (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status`] module"]
#[doc(alias = "STATUS")]
pub type Status = crate::Reg<status::StatusSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter"]
pub mod status;
#[doc = "LOCK (w) register accessor: no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lock::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lock`] module"]
#[doc(alias = "LOCK")]
pub type Lock = crate::Reg<lock::LockSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, DPLL chapter"]
pub mod lock;
