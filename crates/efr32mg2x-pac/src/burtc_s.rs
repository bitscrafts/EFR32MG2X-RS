//! Back-Up Real Time Counter. Registers of `BURTC_S`.
#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    ipversion: Ipversion,
    en: En,
    cfg: Cfg,
    cmd: Cmd,
    status: Status,
    if_: If,
    ien: Ien,
    precnt: Precnt,
    cnt: Cnt,
    em4wuen: Em4wuen,
    syncbusy: Syncbusy,
    lock: Lock,
    comp: Comp,
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
    pub const fn cmd(&self) -> &Cmd {
        &self.cmd
    }
    #[doc = "0x10 - register without a vendor description; see the Reference Manual"]
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
    #[doc = "0x1c - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn precnt(&self) -> &Precnt {
        &self.precnt
    }
    #[doc = "0x20 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn cnt(&self) -> &Cnt {
        &self.cnt
    }
    #[doc = "0x24 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn em4wuen(&self) -> &Em4wuen {
        &self.em4wuen
    }
    #[doc = "0x28 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn syncbusy(&self) -> &Syncbusy {
        &self.syncbusy
    }
    #[doc = "0x2c - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn lock(&self) -> &Lock {
        &self.lock
    }
    #[doc = "0x30 - register without a vendor description; see the Reference Manual"]
    #[inline(always)]
    pub const fn comp(&self) -> &Comp {
        &self.comp
    }
}
#[doc = "IPVERSION (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ipversion::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ipversion`] module"]
#[doc(alias = "IPVERSION")]
pub type Ipversion = crate::Reg<ipversion::IpversionSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod ipversion;
#[doc = "EN (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`en::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`en::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@en`] module"]
#[doc(alias = "EN")]
pub type En = crate::Reg<en::EnSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod en;
#[doc = "CFG (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cfg`] module"]
#[doc(alias = "CFG")]
pub type Cfg = crate::Reg<cfg::CfgSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod cfg;
#[doc = "CMD (w) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmd::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmd`] module"]
#[doc(alias = "CMD")]
pub type Cmd = crate::Reg<cmd::CmdSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod cmd;
#[doc = "STATUS (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status`] module"]
#[doc(alias = "STATUS")]
pub type Status = crate::Reg<status::StatusSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod status;
#[doc = "IF (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`if_::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`if_::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@if_`] module"]
#[doc(alias = "IF")]
pub type If = crate::Reg<if_::IfSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod if_;
#[doc = "IEN (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`ien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ien`] module"]
#[doc(alias = "IEN")]
pub type Ien = crate::Reg<ien::IenSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod ien;
#[doc = "PRECNT (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`precnt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`precnt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@precnt`] module"]
#[doc(alias = "PRECNT")]
pub type Precnt = crate::Reg<precnt::PrecntSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod precnt;
#[doc = "CNT (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`cnt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cnt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cnt`] module"]
#[doc(alias = "CNT")]
pub type Cnt = crate::Reg<cnt::CntSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod cnt;
#[doc = "EM4WUEN (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`em4wuen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`em4wuen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@em4wuen`] module"]
#[doc(alias = "EM4WUEN")]
pub type Em4wuen = crate::Reg<em4wuen::Em4wuenSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod em4wuen;
#[doc = "SYNCBUSY (r) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`syncbusy::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@syncbusy`] module"]
#[doc(alias = "SYNCBUSY")]
pub type Syncbusy = crate::Reg<syncbusy::SyncbusySpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod syncbusy;
#[doc = "LOCK (w) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lock::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@lock`] module"]
#[doc(alias = "LOCK")]
pub type Lock = crate::Reg<lock::LockSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod lock;
#[doc = "COMP (rw) register accessor: no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter\n\nYou can [`read`](crate::Reg::read) this register and get [`comp::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`comp::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@comp`] module"]
#[doc(alias = "COMP")]
pub type Comp = crate::Reg<comp::CompSpec>;
#[doc = "no vendor description; see the EFR32xG24 Reference Manual, BURTC chapter"]
pub mod comp;
