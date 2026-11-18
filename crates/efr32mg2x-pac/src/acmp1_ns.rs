//! Analog Comparator. Registers of `ACMP1_NS`.
#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    ipversion: Ipversion,
    en: En,
    swrst: Swrst,
    cfg: Cfg,
    ctrl: Ctrl,
    inputctrl: Inputctrl,
    status: Status,
    if_: If,
    ien: Ien,
    syncbusy: Syncbusy,
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
    pub const fn swrst(&self) -> &Swrst {
        &self.swrst
    }
    #[doc = "0x0c - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn cfg(&self) -> &Cfg {
        &self.cfg
    }
    #[doc = "0x10 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ctrl(&self) -> &Ctrl {
        &self.ctrl
    }
    #[doc = "0x14 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn inputctrl(&self) -> &Inputctrl {
        &self.inputctrl
    }
    #[doc = "0x18 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn status(&self) -> &Status {
        &self.status
    }
    #[doc = "0x1c - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn if_(&self) -> &If {
        &self.if_
    }
    #[doc = "0x20 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ien(&self) -> &Ien {
        &self.ien
    }
    #[doc = "0x24 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn syncbusy(&self) -> &Syncbusy {
        &self.syncbusy
    }
}
#[doc = "IPVERSION (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ipversion::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ipversion`] module"]
#[doc(alias = "IPVERSION")]
pub type Ipversion = crate::Reg<ipversion::IpversionSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter"]
pub mod ipversion;
#[doc = "EN (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`en::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`en::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@en`] module"]
#[doc(alias = "EN")]
pub type En = crate::Reg<en::EnSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter"]
pub mod en;
#[doc = "SWRST (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`swrst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swrst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@swrst`] module"]
#[doc(alias = "SWRST")]
pub type Swrst = crate::Reg<swrst::SwrstSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter"]
pub mod swrst;
#[doc = "CFG (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cfg`] module"]
#[doc(alias = "CFG")]
pub type Cfg = crate::Reg<cfg::CfgSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter"]
pub mod cfg;
#[doc = "CTRL (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl`] module"]
#[doc(alias = "CTRL")]
pub type Ctrl = crate::Reg<ctrl::CtrlSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter"]
pub mod ctrl;
#[doc = "INPUTCTRL (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`inputctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`inputctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@inputctrl`] module"]
#[doc(alias = "INPUTCTRL")]
pub type Inputctrl = crate::Reg<inputctrl::InputctrlSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter"]
pub mod inputctrl;
#[doc = "STATUS (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status`] module"]
#[doc(alias = "STATUS")]
pub type Status = crate::Reg<status::StatusSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter"]
pub mod status;
#[doc = "IF (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`if_::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`if_::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@if_`] module"]
#[doc(alias = "IF")]
pub type If = crate::Reg<if_::IfSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter"]
pub mod if_;
#[doc = "IEN (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ien`] module"]
#[doc(alias = "IEN")]
pub type Ien = crate::Reg<ien::IenSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter"]
pub mod ien;
#[doc = "SYNCBUSY (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`syncbusy::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@syncbusy`] module"]
#[doc(alias = "SYNCBUSY")]
pub type Syncbusy = crate::Reg<syncbusy::SyncbusySpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, ACMP chapter"]
pub mod syncbusy;
