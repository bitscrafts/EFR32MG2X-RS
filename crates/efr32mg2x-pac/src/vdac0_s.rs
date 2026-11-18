//! Digital to Analog Converter. Registers of `VDAC0_S`.
#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    ipversion: Ipversion,
    en: En,
    swrst: Swrst,
    cfg: Cfg,
    status: Status,
    ch0cfg: Ch0cfg,
    ch1cfg: Ch1cfg,
    cmd: Cmd,
    if_: If,
    ien: Ien,
    ch0f: Ch0f,
    ch1f: Ch1f,
    outctrl: Outctrl,
    outtimercfg: Outtimercfg,
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
    pub const fn status(&self) -> &Status {
        &self.status
    }
    #[doc = "0x14 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ch0cfg(&self) -> &Ch0cfg {
        &self.ch0cfg
    }
    #[doc = "0x18 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ch1cfg(&self) -> &Ch1cfg {
        &self.ch1cfg
    }
    #[doc = "0x1c - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn cmd(&self) -> &Cmd {
        &self.cmd
    }
    #[doc = "0x20 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn if_(&self) -> &If {
        &self.if_
    }
    #[doc = "0x24 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ien(&self) -> &Ien {
        &self.ien
    }
    #[doc = "0x28 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ch0f(&self) -> &Ch0f {
        &self.ch0f
    }
    #[doc = "0x2c - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn ch1f(&self) -> &Ch1f {
        &self.ch1f
    }
    #[doc = "0x30 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn outctrl(&self) -> &Outctrl {
        &self.outctrl
    }
    #[doc = "0x34 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn outtimercfg(&self) -> &Outtimercfg {
        &self.outtimercfg
    }
}
#[doc = "IPVERSION (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ipversion::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ipversion`] module"]
#[doc(alias = "IPVERSION")]
pub type Ipversion = crate::Reg<ipversion::IpversionSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod ipversion;
#[doc = "EN (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`en::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`en::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@en`] module"]
#[doc(alias = "EN")]
pub type En = crate::Reg<en::EnSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod en;
#[doc = "SWRST (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`swrst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swrst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@swrst`] module"]
#[doc(alias = "SWRST")]
pub type Swrst = crate::Reg<swrst::SwrstSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod swrst;
#[doc = "CFG (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cfg`] module"]
#[doc(alias = "CFG")]
pub type Cfg = crate::Reg<cfg::CfgSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod cfg;
#[doc = "STATUS (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status`] module"]
#[doc(alias = "STATUS")]
pub type Status = crate::Reg<status::StatusSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod status;
#[doc = "CH0CFG (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ch0cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch0cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch0cfg`] module"]
#[doc(alias = "CH0CFG")]
pub type Ch0cfg = crate::Reg<ch0cfg::Ch0cfgSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod ch0cfg;
#[doc = "CH1CFG (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ch1cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch1cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch1cfg`] module"]
#[doc(alias = "CH1CFG")]
pub type Ch1cfg = crate::Reg<ch1cfg::Ch1cfgSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod ch1cfg;
#[doc = "CMD (w) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmd::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmd`] module"]
#[doc(alias = "CMD")]
pub type Cmd = crate::Reg<cmd::CmdSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod cmd;
#[doc = "IF (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`if_::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`if_::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@if_`] module"]
#[doc(alias = "IF")]
pub type If = crate::Reg<if_::IfSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod if_;
#[doc = "IEN (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ien`] module"]
#[doc(alias = "IEN")]
pub type Ien = crate::Reg<ien::IenSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod ien;
#[doc = "CH0F (w) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch0f::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch0f`] module"]
#[doc(alias = "CH0F")]
pub type Ch0f = crate::Reg<ch0f::Ch0fSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod ch0f;
#[doc = "CH1F (w) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch1f::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ch1f`] module"]
#[doc(alias = "CH1F")]
pub type Ch1f = crate::Reg<ch1f::Ch1fSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod ch1f;
#[doc = "OUTCTRL (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`outctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`outctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@outctrl`] module"]
#[doc(alias = "OUTCTRL")]
pub type Outctrl = crate::Reg<outctrl::OutctrlSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod outctrl;
#[doc = "OUTTIMERCFG (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`outtimercfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`outtimercfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@outtimercfg`] module"]
#[doc(alias = "OUTTIMERCFG")]
pub type Outtimercfg = crate::Reg<outtimercfg::OuttimercfgSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, VDAC chapter"]
pub mod outtimercfg;
