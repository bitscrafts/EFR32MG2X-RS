#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    fetchaddr: Fetchaddr,
    _reserved1: [u8; 0x04],
    fetchlen: Fetchlen,
    fetchtag: Fetchtag,
    pushaddr: Pushaddr,
    _reserved4: [u8; 0x04],
    pushlen: Pushlen,
    ien: Ien,
    _reserved6: [u8; 0x08],
    if_: If,
    _reserved7: [u8; 0x04],
    if_clr: IfClr,
    ctrl: Ctrl,
    cmd: Cmd,
    status: Status,
    _reserved11: [u8; 0x03c0],
    incl_ips_hw_cfg: InclIpsHwCfg,
    ba411e_hw_cfg_1: Ba411eHwCfg1,
    ba411e_hw_cfg_2: Ba411eHwCfg2,
    ba413_hw_cfg: Ba413HwCfg,
    ba418_hw_cfg: Ba418HwCfg,
    ba419_hw_cfg: Ba419HwCfg,
}
impl RegisterBlock {
    #[doc = "0x00 - Fetcher: Start address of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is updated after each processed descriptor."]
    #[inline(always)]
    pub const fn fetchaddr(&self) -> &Fetchaddr {
        &self.fetchaddr
    }
    #[doc = "0x08 - Fetcher: Length of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used."]
    #[inline(always)]
    pub const fn fetchlen(&self) -> &Fetchlen {
        &self.fetchlen
    }
    #[doc = "0x0c - Fetcher: User tag. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used."]
    #[inline(always)]
    pub const fn fetchtag(&self) -> &Fetchtag {
        &self.fetchtag
    }
    #[doc = "0x10 - Pusher: Start address of data block (LSB). In direct mode, this register is written by the software. In scatter-gather mode, this register is updated after each processed descriptor."]
    #[inline(always)]
    pub const fn pushaddr(&self) -> &Pushaddr {
        &self.pushaddr
    }
    #[doc = "0x18 - Pusher: Length of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used."]
    #[inline(always)]
    pub const fn pushlen(&self) -> &Pushlen {
        &self.pushlen
    }
    #[doc = "0x1c - Interrupt enable"]
    #[inline(always)]
    pub const fn ien(&self) -> &Ien {
        &self.ien
    }
    #[doc = "0x28 - Interrupt flag register"]
    #[inline(always)]
    pub const fn if_(&self) -> &If {
        &self.if_
    }
    #[doc = "0x30 - Writing a '1' clears the interrupt status. Writing a '0' has no effect."]
    #[inline(always)]
    pub const fn if_clr(&self) -> &IfClr {
        &self.if_clr
    }
    #[doc = "0x34 - Control register, called CONFIG in Barco datasheet."]
    #[inline(always)]
    pub const fn ctrl(&self) -> &Ctrl {
        &self.ctrl
    }
    #[doc = "0x38 - Command register for starting the fetcher and pusher"]
    #[inline(always)]
    pub const fn cmd(&self) -> &Cmd {
        &self.cmd
    }
    #[doc = "0x3c - Status register"]
    #[inline(always)]
    pub const fn status(&self) -> &Status {
        &self.status
    }
    #[doc = "0x400 - No Description"]
    #[inline(always)]
    pub const fn incl_ips_hw_cfg(&self) -> &InclIpsHwCfg {
        &self.incl_ips_hw_cfg
    }
    #[doc = "0x404 - No Description"]
    #[inline(always)]
    pub const fn ba411e_hw_cfg_1(&self) -> &Ba411eHwCfg1 {
        &self.ba411e_hw_cfg_1
    }
    #[doc = "0x408 - No Description"]
    #[inline(always)]
    pub const fn ba411e_hw_cfg_2(&self) -> &Ba411eHwCfg2 {
        &self.ba411e_hw_cfg_2
    }
    #[doc = "0x40c - No Description"]
    #[inline(always)]
    pub const fn ba413_hw_cfg(&self) -> &Ba413HwCfg {
        &self.ba413_hw_cfg
    }
    #[doc = "0x410 - No Description"]
    #[inline(always)]
    pub const fn ba418_hw_cfg(&self) -> &Ba418HwCfg {
        &self.ba418_hw_cfg
    }
    #[doc = "0x414 - No Description"]
    #[inline(always)]
    pub const fn ba419_hw_cfg(&self) -> &Ba419HwCfg {
        &self.ba419_hw_cfg
    }
}
#[doc = "FETCHADDR (rw) register accessor: Fetcher: Start address of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is updated after each processed descriptor.\n\nYou can [`read`](crate::Reg::read) this register and get [`fetchaddr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fetchaddr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fetchaddr`] module"]
#[doc(alias = "FETCHADDR")]
pub type Fetchaddr = crate::Reg<fetchaddr::FetchaddrSpec>;
#[doc = "Fetcher: Start address of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is updated after each processed descriptor."]
pub mod fetchaddr;
#[doc = "FETCHLEN (rw) register accessor: Fetcher: Length of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used.\n\nYou can [`read`](crate::Reg::read) this register and get [`fetchlen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fetchlen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fetchlen`] module"]
#[doc(alias = "FETCHLEN")]
pub type Fetchlen = crate::Reg<fetchlen::FetchlenSpec>;
#[doc = "Fetcher: Length of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used."]
pub mod fetchlen;
#[doc = "FETCHTAG (rw) register accessor: Fetcher: User tag. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used.\n\nYou can [`read`](crate::Reg::read) this register and get [`fetchtag::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fetchtag::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fetchtag`] module"]
#[doc(alias = "FETCHTAG")]
pub type Fetchtag = crate::Reg<fetchtag::FetchtagSpec>;
#[doc = "Fetcher: User tag. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used."]
pub mod fetchtag;
#[doc = "PUSHADDR (rw) register accessor: Pusher: Start address of data block (LSB). In direct mode, this register is written by the software. In scatter-gather mode, this register is updated after each processed descriptor.\n\nYou can [`read`](crate::Reg::read) this register and get [`pushaddr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pushaddr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pushaddr`] module"]
#[doc(alias = "PUSHADDR")]
pub type Pushaddr = crate::Reg<pushaddr::PushaddrSpec>;
#[doc = "Pusher: Start address of data block (LSB). In direct mode, this register is written by the software. In scatter-gather mode, this register is updated after each processed descriptor."]
pub mod pushaddr;
#[doc = "PUSHLEN (rw) register accessor: Pusher: Length of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used.\n\nYou can [`read`](crate::Reg::read) this register and get [`pushlen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pushlen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pushlen`] module"]
#[doc(alias = "PUSHLEN")]
pub type Pushlen = crate::Reg<pushlen::PushlenSpec>;
#[doc = "Pusher: Length of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used."]
pub mod pushlen;
#[doc = "IEN (rw) register accessor: Interrupt enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ien`] module"]
#[doc(alias = "IEN")]
pub type Ien = crate::Reg<ien::IenSpec>;
#[doc = "Interrupt enable"]
pub mod ien;
#[doc = "IF (r) register accessor: Interrupt flag register\n\nYou can [`read`](crate::Reg::read) this register and get [`if_::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@if_`] module"]
#[doc(alias = "IF")]
pub type If = crate::Reg<if_::IfSpec>;
#[doc = "Interrupt flag register"]
pub mod if_;
#[doc = "IF_CLR (w) register accessor: Writing a '1' clears the interrupt status. Writing a '0' has no effect.\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`if_clr::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@if_clr`] module"]
#[doc(alias = "IF_CLR")]
pub type IfClr = crate::Reg<if_clr::IfClrSpec>;
#[doc = "Writing a '1' clears the interrupt status. Writing a '0' has no effect."]
pub mod if_clr;
#[doc = "CTRL (rw) register accessor: Control register, called CONFIG in Barco datasheet.\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ctrl`] module"]
#[doc(alias = "CTRL")]
pub type Ctrl = crate::Reg<ctrl::CtrlSpec>;
#[doc = "Control register, called CONFIG in Barco datasheet."]
pub mod ctrl;
#[doc = "CMD (w) register accessor: Command register for starting the fetcher and pusher\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmd::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmd`] module"]
#[doc(alias = "CMD")]
pub type Cmd = crate::Reg<cmd::CmdSpec>;
#[doc = "Command register for starting the fetcher and pusher"]
pub mod cmd;
#[doc = "STATUS (r) register accessor: Status register\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status`] module"]
#[doc(alias = "STATUS")]
pub type Status = crate::Reg<status::StatusSpec>;
#[doc = "Status register"]
pub mod status;
#[doc = "INCL_IPS_HW_CFG (r) register accessor: No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`incl_ips_hw_cfg::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@incl_ips_hw_cfg`] module"]
#[doc(alias = "INCL_IPS_HW_CFG")]
pub type InclIpsHwCfg = crate::Reg<incl_ips_hw_cfg::InclIpsHwCfgSpec>;
#[doc = "No Description"]
pub mod incl_ips_hw_cfg;
#[doc = "BA411E_HW_CFG_1 (r) register accessor: No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`ba411e_hw_cfg_1::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ba411e_hw_cfg_1`] module"]
#[doc(alias = "BA411E_HW_CFG_1")]
pub type Ba411eHwCfg1 = crate::Reg<ba411e_hw_cfg_1::Ba411eHwCfg1Spec>;
#[doc = "No Description"]
pub mod ba411e_hw_cfg_1;
#[doc = "BA411E_HW_CFG_2 (r) register accessor: No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`ba411e_hw_cfg_2::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ba411e_hw_cfg_2`] module"]
#[doc(alias = "BA411E_HW_CFG_2")]
pub type Ba411eHwCfg2 = crate::Reg<ba411e_hw_cfg_2::Ba411eHwCfg2Spec>;
#[doc = "No Description"]
pub mod ba411e_hw_cfg_2;
#[doc = "BA413_HW_CFG (r) register accessor: No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`ba413_hw_cfg::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ba413_hw_cfg`] module"]
#[doc(alias = "BA413_HW_CFG")]
pub type Ba413HwCfg = crate::Reg<ba413_hw_cfg::Ba413HwCfgSpec>;
#[doc = "No Description"]
pub mod ba413_hw_cfg;
#[doc = "BA418_HW_CFG (r) register accessor: No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`ba418_hw_cfg::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ba418_hw_cfg`] module"]
#[doc(alias = "BA418_HW_CFG")]
pub type Ba418HwCfg = crate::Reg<ba418_hw_cfg::Ba418HwCfgSpec>;
#[doc = "No Description"]
pub mod ba418_hw_cfg;
#[doc = "BA419_HW_CFG (r) register accessor: No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`ba419_hw_cfg::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ba419_hw_cfg`] module"]
#[doc(alias = "BA419_HW_CFG")]
pub type Ba419HwCfg = crate::Reg<ba419_hw_cfg::Ba419HwCfgSpec>;
#[doc = "No Description"]
pub mod ba419_hw_cfg;
