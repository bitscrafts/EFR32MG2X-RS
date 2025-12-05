#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    ipversion: Ipversion,
    en: En,
    swrst: Swrst,
    cfg: Cfg,
    status: Status,
    perf0cnt: Perf0cnt,
    perf1cnt: Perf1cnt,
    if_: If,
    ien: Ien,
    faultstatus: Faultstatus,
    faultaddr: Faultaddr,
    programstate: Programstate,
    array0indexstate: Array0indexstate,
    array1indexstate: Array1indexstate,
    array2indexstate: Array2indexstate,
    array3indexstate: Array3indexstate,
    array4indexstate: Array4indexstate,
    loop0state: Loop0state,
    loop1state: Loop1state,
    loop2state: Loop2state,
    loop3state: Loop3state,
    loop4state: Loop4state,
    loop5state: Loop5state,
    loop6state: Loop6state,
    loop7state: Loop7state,
    alu0regstate: Alu0regstate,
    alu1regstate: Alu1regstate,
    alu2regstate: Alu2regstate,
    alu3regstate: Alu3regstate,
    alu4regstate: Alu4regstate,
    alu5regstate: Alu5regstate,
    alu6regstate: Alu6regstate,
    alu7regstate: Alu7regstate,
    array0addrcfg: Array0addrcfg,
    array0dim0cfg: Array0dim0cfg,
    array0dim1cfg: Array0dim1cfg,
    array0dim2cfg: Array0dim2cfg,
    array1addrcfg: Array1addrcfg,
    array1dim0cfg: Array1dim0cfg,
    array1dim1cfg: Array1dim1cfg,
    array1dim2cfg: Array1dim2cfg,
    array2addrcfg: Array2addrcfg,
    array2dim0cfg: Array2dim0cfg,
    array2dim1cfg: Array2dim1cfg,
    array2dim2cfg: Array2dim2cfg,
    array3addrcfg: Array3addrcfg,
    array3dim0cfg: Array3dim0cfg,
    array3dim1cfg: Array3dim1cfg,
    array3dim2cfg: Array3dim2cfg,
    array4addrcfg: Array4addrcfg,
    array4dim0cfg: Array4dim0cfg,
    array4dim1cfg: Array4dim1cfg,
    array4dim2cfg: Array4dim2cfg,
    loop0cfg: Loop0cfg,
    loop0rst: Loop0rst,
    loop1cfg: Loop1cfg,
    loop1rst: Loop1rst,
    loop2cfg: Loop2cfg,
    loop2rst: Loop2rst,
    loop3cfg: Loop3cfg,
    loop3rst: Loop3rst,
    loop4cfg: Loop4cfg,
    loop4rst: Loop4rst,
    loop5cfg: Loop5cfg,
    loop5rst: Loop5rst,
    loop6cfg: Loop6cfg,
    loop6rst: Loop6rst,
    loop7cfg: Loop7cfg,
    loop7rst: Loop7rst,
    instr0cfg0: Instr0cfg0,
    instr0cfg1: Instr0cfg1,
    instr0cfg2: Instr0cfg2,
    instr1cfg0: Instr1cfg0,
    instr1cfg1: Instr1cfg1,
    instr1cfg2: Instr1cfg2,
    instr2cfg0: Instr2cfg0,
    instr2cfg1: Instr2cfg1,
    instr2cfg2: Instr2cfg2,
    instr3cfg0: Instr3cfg0,
    instr3cfg1: Instr3cfg1,
    instr3cfg2: Instr3cfg2,
    instr4cfg0: Instr4cfg0,
    instr4cfg1: Instr4cfg1,
    instr4cfg2: Instr4cfg2,
    instr5cfg0: Instr5cfg0,
    instr5cfg1: Instr5cfg1,
    instr5cfg2: Instr5cfg2,
    instr6cfg0: Instr6cfg0,
    instr6cfg1: Instr6cfg1,
    instr6cfg2: Instr6cfg2,
    instr7cfg0: Instr7cfg0,
    instr7cfg1: Instr7cfg1,
    instr7cfg2: Instr7cfg2,
    cmd: Cmd,
    _reserved94: [u8; 0x88],
    debugen: Debugen,
    debugstepcnt: Debugstepcnt,
}
impl RegisterBlock {
    #[doc = "0x00 - IP Version Register"]
    #[inline(always)]
    pub const fn ipversion(&self) -> &Ipversion {
        &self.ipversion
    }
    #[doc = "0x04 - Block Enable Register"]
    #[inline(always)]
    pub const fn en(&self) -> &En {
        &self.en
    }
    #[doc = "0x08 - Software Reset Register"]
    #[inline(always)]
    pub const fn swrst(&self) -> &Swrst {
        &self.swrst
    }
    #[doc = "0x0c - Configuration Register"]
    #[inline(always)]
    pub const fn cfg(&self) -> &Cfg {
        &self.cfg
    }
    #[doc = "0x10 - Status Register"]
    #[inline(always)]
    pub const fn status(&self) -> &Status {
        &self.status
    }
    #[doc = "0x14 - Run Counter"]
    #[inline(always)]
    pub const fn perf0cnt(&self) -> &Perf0cnt {
        &self.perf0cnt
    }
    #[doc = "0x18 - Run Counter"]
    #[inline(always)]
    pub const fn perf1cnt(&self) -> &Perf1cnt {
        &self.perf1cnt
    }
    #[doc = "0x1c - Interrupt Flags"]
    #[inline(always)]
    pub const fn if_(&self) -> &If {
        &self.if_
    }
    #[doc = "0x20 - Interrupt Enable"]
    #[inline(always)]
    pub const fn ien(&self) -> &Ien {
        &self.ien
    }
    #[doc = "0x24 - Fault Status Register"]
    #[inline(always)]
    pub const fn faultstatus(&self) -> &Faultstatus {
        &self.faultstatus
    }
    #[doc = "0x28 - Fault Address Register"]
    #[inline(always)]
    pub const fn faultaddr(&self) -> &Faultaddr {
        &self.faultaddr
    }
    #[doc = "0x2c - Program State Register"]
    #[inline(always)]
    pub const fn programstate(&self) -> &Programstate {
        &self.programstate
    }
    #[doc = "0x30 - Array N Index State Register"]
    #[inline(always)]
    pub const fn array0indexstate(&self) -> &Array0indexstate {
        &self.array0indexstate
    }
    #[doc = "0x34 - Array N Index State Register"]
    #[inline(always)]
    pub const fn array1indexstate(&self) -> &Array1indexstate {
        &self.array1indexstate
    }
    #[doc = "0x38 - Array N Index State Register"]
    #[inline(always)]
    pub const fn array2indexstate(&self) -> &Array2indexstate {
        &self.array2indexstate
    }
    #[doc = "0x3c - Array N Index State Register"]
    #[inline(always)]
    pub const fn array3indexstate(&self) -> &Array3indexstate {
        &self.array3indexstate
    }
    #[doc = "0x40 - Array N Index State Register"]
    #[inline(always)]
    pub const fn array4indexstate(&self) -> &Array4indexstate {
        &self.array4indexstate
    }
    #[doc = "0x44 - Loop N State Register"]
    #[inline(always)]
    pub const fn loop0state(&self) -> &Loop0state {
        &self.loop0state
    }
    #[doc = "0x48 - Loop N State Register"]
    #[inline(always)]
    pub const fn loop1state(&self) -> &Loop1state {
        &self.loop1state
    }
    #[doc = "0x4c - Loop N State Register"]
    #[inline(always)]
    pub const fn loop2state(&self) -> &Loop2state {
        &self.loop2state
    }
    #[doc = "0x50 - Loop N State Register"]
    #[inline(always)]
    pub const fn loop3state(&self) -> &Loop3state {
        &self.loop3state
    }
    #[doc = "0x54 - Loop N State Register"]
    #[inline(always)]
    pub const fn loop4state(&self) -> &Loop4state {
        &self.loop4state
    }
    #[doc = "0x58 - Loop N State Register"]
    #[inline(always)]
    pub const fn loop5state(&self) -> &Loop5state {
        &self.loop5state
    }
    #[doc = "0x5c - Loop N State Register"]
    #[inline(always)]
    pub const fn loop6state(&self) -> &Loop6state {
        &self.loop6state
    }
    #[doc = "0x60 - Loop N State Register"]
    #[inline(always)]
    pub const fn loop7state(&self) -> &Loop7state {
        &self.loop7state
    }
    #[doc = "0x64 - ALU Register"]
    #[inline(always)]
    pub const fn alu0regstate(&self) -> &Alu0regstate {
        &self.alu0regstate
    }
    #[doc = "0x68 - ALU Register"]
    #[inline(always)]
    pub const fn alu1regstate(&self) -> &Alu1regstate {
        &self.alu1regstate
    }
    #[doc = "0x6c - ALU Register"]
    #[inline(always)]
    pub const fn alu2regstate(&self) -> &Alu2regstate {
        &self.alu2regstate
    }
    #[doc = "0x70 - ALU Register"]
    #[inline(always)]
    pub const fn alu3regstate(&self) -> &Alu3regstate {
        &self.alu3regstate
    }
    #[doc = "0x74 - ALU Register"]
    #[inline(always)]
    pub const fn alu4regstate(&self) -> &Alu4regstate {
        &self.alu4regstate
    }
    #[doc = "0x78 - ALU Register"]
    #[inline(always)]
    pub const fn alu5regstate(&self) -> &Alu5regstate {
        &self.alu5regstate
    }
    #[doc = "0x7c - ALU Register"]
    #[inline(always)]
    pub const fn alu6regstate(&self) -> &Alu6regstate {
        &self.alu6regstate
    }
    #[doc = "0x80 - ALU Register"]
    #[inline(always)]
    pub const fn alu7regstate(&self) -> &Alu7regstate {
        &self.alu7regstate
    }
    #[doc = "0x84 - Array N Base Address Register"]
    #[inline(always)]
    pub const fn array0addrcfg(&self) -> &Array0addrcfg {
        &self.array0addrcfg
    }
    #[doc = "0x88 - Array N Dimenion 0 Configuration"]
    #[inline(always)]
    pub const fn array0dim0cfg(&self) -> &Array0dim0cfg {
        &self.array0dim0cfg
    }
    #[doc = "0x8c - Array N Dimenion 1 Configuration"]
    #[inline(always)]
    pub const fn array0dim1cfg(&self) -> &Array0dim1cfg {
        &self.array0dim1cfg
    }
    #[doc = "0x90 - Array N Dimenion 2 Configuration"]
    #[inline(always)]
    pub const fn array0dim2cfg(&self) -> &Array0dim2cfg {
        &self.array0dim2cfg
    }
    #[doc = "0x94 - Array N Base Address Register"]
    #[inline(always)]
    pub const fn array1addrcfg(&self) -> &Array1addrcfg {
        &self.array1addrcfg
    }
    #[doc = "0x98 - Array N Dimenion 0 Configuration"]
    #[inline(always)]
    pub const fn array1dim0cfg(&self) -> &Array1dim0cfg {
        &self.array1dim0cfg
    }
    #[doc = "0x9c - Array N Dimenion 1 Configuration"]
    #[inline(always)]
    pub const fn array1dim1cfg(&self) -> &Array1dim1cfg {
        &self.array1dim1cfg
    }
    #[doc = "0xa0 - Array N Dimenion 2 Configuration"]
    #[inline(always)]
    pub const fn array1dim2cfg(&self) -> &Array1dim2cfg {
        &self.array1dim2cfg
    }
    #[doc = "0xa4 - Array N Base Address Register"]
    #[inline(always)]
    pub const fn array2addrcfg(&self) -> &Array2addrcfg {
        &self.array2addrcfg
    }
    #[doc = "0xa8 - Array N Dimenion 0 Configuration"]
    #[inline(always)]
    pub const fn array2dim0cfg(&self) -> &Array2dim0cfg {
        &self.array2dim0cfg
    }
    #[doc = "0xac - Array N Dimenion 1 Configuration"]
    #[inline(always)]
    pub const fn array2dim1cfg(&self) -> &Array2dim1cfg {
        &self.array2dim1cfg
    }
    #[doc = "0xb0 - Array N Dimenion 2 Configuration"]
    #[inline(always)]
    pub const fn array2dim2cfg(&self) -> &Array2dim2cfg {
        &self.array2dim2cfg
    }
    #[doc = "0xb4 - Array N Base Address Register"]
    #[inline(always)]
    pub const fn array3addrcfg(&self) -> &Array3addrcfg {
        &self.array3addrcfg
    }
    #[doc = "0xb8 - Array N Dimenion 0 Configuration"]
    #[inline(always)]
    pub const fn array3dim0cfg(&self) -> &Array3dim0cfg {
        &self.array3dim0cfg
    }
    #[doc = "0xbc - Array N Dimenion 1 Configuration"]
    #[inline(always)]
    pub const fn array3dim1cfg(&self) -> &Array3dim1cfg {
        &self.array3dim1cfg
    }
    #[doc = "0xc0 - Array N Dimenion 2 Configuration"]
    #[inline(always)]
    pub const fn array3dim2cfg(&self) -> &Array3dim2cfg {
        &self.array3dim2cfg
    }
    #[doc = "0xc4 - Array N Base Address Register"]
    #[inline(always)]
    pub const fn array4addrcfg(&self) -> &Array4addrcfg {
        &self.array4addrcfg
    }
    #[doc = "0xc8 - Array N Dimenion 0 Configuration"]
    #[inline(always)]
    pub const fn array4dim0cfg(&self) -> &Array4dim0cfg {
        &self.array4dim0cfg
    }
    #[doc = "0xcc - Array N Dimenion 1 Configuration"]
    #[inline(always)]
    pub const fn array4dim1cfg(&self) -> &Array4dim1cfg {
        &self.array4dim1cfg
    }
    #[doc = "0xd0 - Array N Dimenion 2 Configuration"]
    #[inline(always)]
    pub const fn array4dim2cfg(&self) -> &Array4dim2cfg {
        &self.array4dim2cfg
    }
    #[doc = "0xd4 - Loop N Configuration Register"]
    #[inline(always)]
    pub const fn loop0cfg(&self) -> &Loop0cfg {
        &self.loop0cfg
    }
    #[doc = "0xd8 - Loop N Reset Configuration Register"]
    #[inline(always)]
    pub const fn loop0rst(&self) -> &Loop0rst {
        &self.loop0rst
    }
    #[doc = "0xdc - Loop N Configuration Register"]
    #[inline(always)]
    pub const fn loop1cfg(&self) -> &Loop1cfg {
        &self.loop1cfg
    }
    #[doc = "0xe0 - Loop N Reset Configuration Register"]
    #[inline(always)]
    pub const fn loop1rst(&self) -> &Loop1rst {
        &self.loop1rst
    }
    #[doc = "0xe4 - Loop N Configuration Register"]
    #[inline(always)]
    pub const fn loop2cfg(&self) -> &Loop2cfg {
        &self.loop2cfg
    }
    #[doc = "0xe8 - Loop N Reset Configuration Register"]
    #[inline(always)]
    pub const fn loop2rst(&self) -> &Loop2rst {
        &self.loop2rst
    }
    #[doc = "0xec - Loop N Configuration Register"]
    #[inline(always)]
    pub const fn loop3cfg(&self) -> &Loop3cfg {
        &self.loop3cfg
    }
    #[doc = "0xf0 - Loop N Reset Configuration Register"]
    #[inline(always)]
    pub const fn loop3rst(&self) -> &Loop3rst {
        &self.loop3rst
    }
    #[doc = "0xf4 - Loop N Configuration Register"]
    #[inline(always)]
    pub const fn loop4cfg(&self) -> &Loop4cfg {
        &self.loop4cfg
    }
    #[doc = "0xf8 - Loop N Reset Configuration Register"]
    #[inline(always)]
    pub const fn loop4rst(&self) -> &Loop4rst {
        &self.loop4rst
    }
    #[doc = "0xfc - Loop N Configuration Register"]
    #[inline(always)]
    pub const fn loop5cfg(&self) -> &Loop5cfg {
        &self.loop5cfg
    }
    #[doc = "0x100 - Loop N Reset Configuration Register"]
    #[inline(always)]
    pub const fn loop5rst(&self) -> &Loop5rst {
        &self.loop5rst
    }
    #[doc = "0x104 - Loop N Configuration Register"]
    #[inline(always)]
    pub const fn loop6cfg(&self) -> &Loop6cfg {
        &self.loop6cfg
    }
    #[doc = "0x108 - Loop N Reset Configuration Register"]
    #[inline(always)]
    pub const fn loop6rst(&self) -> &Loop6rst {
        &self.loop6rst
    }
    #[doc = "0x10c - Loop N Configuration Register"]
    #[inline(always)]
    pub const fn loop7cfg(&self) -> &Loop7cfg {
        &self.loop7cfg
    }
    #[doc = "0x110 - Loop N Reset Configuration Register"]
    #[inline(always)]
    pub const fn loop7rst(&self) -> &Loop7rst {
        &self.loop7rst
    }
    #[doc = "0x114 - Instruction N Word 0"]
    #[inline(always)]
    pub const fn instr0cfg0(&self) -> &Instr0cfg0 {
        &self.instr0cfg0
    }
    #[doc = "0x118 - Instruction N word 1"]
    #[inline(always)]
    pub const fn instr0cfg1(&self) -> &Instr0cfg1 {
        &self.instr0cfg1
    }
    #[doc = "0x11c - Instruction N word 2"]
    #[inline(always)]
    pub const fn instr0cfg2(&self) -> &Instr0cfg2 {
        &self.instr0cfg2
    }
    #[doc = "0x120 - Instruction N Word 0"]
    #[inline(always)]
    pub const fn instr1cfg0(&self) -> &Instr1cfg0 {
        &self.instr1cfg0
    }
    #[doc = "0x124 - Instruction N word 1"]
    #[inline(always)]
    pub const fn instr1cfg1(&self) -> &Instr1cfg1 {
        &self.instr1cfg1
    }
    #[doc = "0x128 - Instruction N word 2"]
    #[inline(always)]
    pub const fn instr1cfg2(&self) -> &Instr1cfg2 {
        &self.instr1cfg2
    }
    #[doc = "0x12c - Instruction N Word 0"]
    #[inline(always)]
    pub const fn instr2cfg0(&self) -> &Instr2cfg0 {
        &self.instr2cfg0
    }
    #[doc = "0x130 - Instruction N word 1"]
    #[inline(always)]
    pub const fn instr2cfg1(&self) -> &Instr2cfg1 {
        &self.instr2cfg1
    }
    #[doc = "0x134 - Instruction N word 2"]
    #[inline(always)]
    pub const fn instr2cfg2(&self) -> &Instr2cfg2 {
        &self.instr2cfg2
    }
    #[doc = "0x138 - Instruction N Word 0"]
    #[inline(always)]
    pub const fn instr3cfg0(&self) -> &Instr3cfg0 {
        &self.instr3cfg0
    }
    #[doc = "0x13c - Instruction N word 1"]
    #[inline(always)]
    pub const fn instr3cfg1(&self) -> &Instr3cfg1 {
        &self.instr3cfg1
    }
    #[doc = "0x140 - Instruction N word 2"]
    #[inline(always)]
    pub const fn instr3cfg2(&self) -> &Instr3cfg2 {
        &self.instr3cfg2
    }
    #[doc = "0x144 - Instruction N Word 0"]
    #[inline(always)]
    pub const fn instr4cfg0(&self) -> &Instr4cfg0 {
        &self.instr4cfg0
    }
    #[doc = "0x148 - Instruction N word 1"]
    #[inline(always)]
    pub const fn instr4cfg1(&self) -> &Instr4cfg1 {
        &self.instr4cfg1
    }
    #[doc = "0x14c - Instruction N word 2"]
    #[inline(always)]
    pub const fn instr4cfg2(&self) -> &Instr4cfg2 {
        &self.instr4cfg2
    }
    #[doc = "0x150 - Instruction N Word 0"]
    #[inline(always)]
    pub const fn instr5cfg0(&self) -> &Instr5cfg0 {
        &self.instr5cfg0
    }
    #[doc = "0x154 - Instruction N word 1"]
    #[inline(always)]
    pub const fn instr5cfg1(&self) -> &Instr5cfg1 {
        &self.instr5cfg1
    }
    #[doc = "0x158 - Instruction N word 2"]
    #[inline(always)]
    pub const fn instr5cfg2(&self) -> &Instr5cfg2 {
        &self.instr5cfg2
    }
    #[doc = "0x15c - Instruction N Word 0"]
    #[inline(always)]
    pub const fn instr6cfg0(&self) -> &Instr6cfg0 {
        &self.instr6cfg0
    }
    #[doc = "0x160 - Instruction N word 1"]
    #[inline(always)]
    pub const fn instr6cfg1(&self) -> &Instr6cfg1 {
        &self.instr6cfg1
    }
    #[doc = "0x164 - Instruction N word 2"]
    #[inline(always)]
    pub const fn instr6cfg2(&self) -> &Instr6cfg2 {
        &self.instr6cfg2
    }
    #[doc = "0x168 - Instruction N Word 0"]
    #[inline(always)]
    pub const fn instr7cfg0(&self) -> &Instr7cfg0 {
        &self.instr7cfg0
    }
    #[doc = "0x16c - Instruction N word 1"]
    #[inline(always)]
    pub const fn instr7cfg1(&self) -> &Instr7cfg1 {
        &self.instr7cfg1
    }
    #[doc = "0x170 - Instruction N word 2"]
    #[inline(always)]
    pub const fn instr7cfg2(&self) -> &Instr7cfg2 {
        &self.instr7cfg2
    }
    #[doc = "0x174 - Command Register"]
    #[inline(always)]
    pub const fn cmd(&self) -> &Cmd {
        &self.cmd
    }
    #[doc = "0x200 - Debug Control Register"]
    #[inline(always)]
    pub const fn debugen(&self) -> &Debugen {
        &self.debugen
    }
    #[doc = "0x204 - No Description"]
    #[inline(always)]
    pub const fn debugstepcnt(&self) -> &Debugstepcnt {
        &self.debugstepcnt
    }
}
#[doc = "IPVERSION (r) register accessor: IP Version Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ipversion::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ipversion`] module"]
#[doc(alias = "IPVERSION")]
pub type Ipversion = crate::Reg<ipversion::IpversionSpec>;
#[doc = "IP Version Register"]
pub mod ipversion;
#[doc = "EN (rw) register accessor: Block Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`en::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`en::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@en`] module"]
#[doc(alias = "EN")]
pub type En = crate::Reg<en::EnSpec>;
#[doc = "Block Enable Register"]
pub mod en;
#[doc = "SWRST (rw) register accessor: Software Reset Register\n\nYou can [`read`](crate::Reg::read) this register and get [`swrst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swrst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@swrst`] module"]
#[doc(alias = "SWRST")]
pub type Swrst = crate::Reg<swrst::SwrstSpec>;
#[doc = "Software Reset Register"]
pub mod swrst;
#[doc = "CFG (rw) register accessor: Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cfg`] module"]
#[doc(alias = "CFG")]
pub type Cfg = crate::Reg<cfg::CfgSpec>;
#[doc = "Configuration Register"]
pub mod cfg;
#[doc = "STATUS (r) register accessor: Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@status`] module"]
#[doc(alias = "STATUS")]
pub type Status = crate::Reg<status::StatusSpec>;
#[doc = "Status Register"]
pub mod status;
#[doc = "PERF0CNT (r) register accessor: Run Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`perf0cnt::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@perf0cnt`] module"]
#[doc(alias = "PERF0CNT")]
pub type Perf0cnt = crate::Reg<perf0cnt::Perf0cntSpec>;
#[doc = "Run Counter"]
pub mod perf0cnt;
#[doc = "PERF1CNT (r) register accessor: Run Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`perf1cnt::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@perf1cnt`] module"]
#[doc(alias = "PERF1CNT")]
pub type Perf1cnt = crate::Reg<perf1cnt::Perf1cntSpec>;
#[doc = "Run Counter"]
pub mod perf1cnt;
#[doc = "IF (rw) register accessor: Interrupt Flags\n\nYou can [`read`](crate::Reg::read) this register and get [`if_::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`if_::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@if_`] module"]
#[doc(alias = "IF")]
pub type If = crate::Reg<if_::IfSpec>;
#[doc = "Interrupt Flags"]
pub mod if_;
#[doc = "IEN (rw) register accessor: Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ien::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ien::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ien`] module"]
#[doc(alias = "IEN")]
pub type Ien = crate::Reg<ien::IenSpec>;
#[doc = "Interrupt Enable"]
pub mod ien;
#[doc = "FAULTSTATUS (r) register accessor: Fault Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`faultstatus::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@faultstatus`] module"]
#[doc(alias = "FAULTSTATUS")]
pub type Faultstatus = crate::Reg<faultstatus::FaultstatusSpec>;
#[doc = "Fault Status Register"]
pub mod faultstatus;
#[doc = "FAULTADDR (r) register accessor: Fault Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`faultaddr::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@faultaddr`] module"]
#[doc(alias = "FAULTADDR")]
pub type Faultaddr = crate::Reg<faultaddr::FaultaddrSpec>;
#[doc = "Fault Address Register"]
pub mod faultaddr;
#[doc = "PROGRAMSTATE (rw) register accessor: Program State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`programstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`programstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@programstate`] module"]
#[doc(alias = "PROGRAMSTATE")]
pub type Programstate = crate::Reg<programstate::ProgramstateSpec>;
#[doc = "Program State Register"]
pub mod programstate;
#[doc = "ARRAY0INDEXSTATE (rw) register accessor: Array N Index State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array0indexstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array0indexstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array0indexstate`] module"]
#[doc(alias = "ARRAY0INDEXSTATE")]
pub type Array0indexstate = crate::Reg<array0indexstate::Array0indexstateSpec>;
#[doc = "Array N Index State Register"]
pub mod array0indexstate;
#[doc = "ARRAY1INDEXSTATE (rw) register accessor: Array N Index State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array1indexstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array1indexstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array1indexstate`] module"]
#[doc(alias = "ARRAY1INDEXSTATE")]
pub type Array1indexstate = crate::Reg<array1indexstate::Array1indexstateSpec>;
#[doc = "Array N Index State Register"]
pub mod array1indexstate;
#[doc = "ARRAY2INDEXSTATE (rw) register accessor: Array N Index State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array2indexstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array2indexstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array2indexstate`] module"]
#[doc(alias = "ARRAY2INDEXSTATE")]
pub type Array2indexstate = crate::Reg<array2indexstate::Array2indexstateSpec>;
#[doc = "Array N Index State Register"]
pub mod array2indexstate;
#[doc = "ARRAY3INDEXSTATE (rw) register accessor: Array N Index State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array3indexstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array3indexstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array3indexstate`] module"]
#[doc(alias = "ARRAY3INDEXSTATE")]
pub type Array3indexstate = crate::Reg<array3indexstate::Array3indexstateSpec>;
#[doc = "Array N Index State Register"]
pub mod array3indexstate;
#[doc = "ARRAY4INDEXSTATE (rw) register accessor: Array N Index State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array4indexstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array4indexstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array4indexstate`] module"]
#[doc(alias = "ARRAY4INDEXSTATE")]
pub type Array4indexstate = crate::Reg<array4indexstate::Array4indexstateSpec>;
#[doc = "Array N Index State Register"]
pub mod array4indexstate;
#[doc = "LOOP0STATE (rw) register accessor: Loop N State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop0state::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop0state::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop0state`] module"]
#[doc(alias = "LOOP0STATE")]
pub type Loop0state = crate::Reg<loop0state::Loop0stateSpec>;
#[doc = "Loop N State Register"]
pub mod loop0state;
#[doc = "LOOP1STATE (rw) register accessor: Loop N State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop1state::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop1state::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop1state`] module"]
#[doc(alias = "LOOP1STATE")]
pub type Loop1state = crate::Reg<loop1state::Loop1stateSpec>;
#[doc = "Loop N State Register"]
pub mod loop1state;
#[doc = "LOOP2STATE (rw) register accessor: Loop N State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop2state::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop2state::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop2state`] module"]
#[doc(alias = "LOOP2STATE")]
pub type Loop2state = crate::Reg<loop2state::Loop2stateSpec>;
#[doc = "Loop N State Register"]
pub mod loop2state;
#[doc = "LOOP3STATE (rw) register accessor: Loop N State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop3state::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop3state::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop3state`] module"]
#[doc(alias = "LOOP3STATE")]
pub type Loop3state = crate::Reg<loop3state::Loop3stateSpec>;
#[doc = "Loop N State Register"]
pub mod loop3state;
#[doc = "LOOP4STATE (rw) register accessor: Loop N State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop4state::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop4state::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop4state`] module"]
#[doc(alias = "LOOP4STATE")]
pub type Loop4state = crate::Reg<loop4state::Loop4stateSpec>;
#[doc = "Loop N State Register"]
pub mod loop4state;
#[doc = "LOOP5STATE (rw) register accessor: Loop N State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop5state::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop5state::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop5state`] module"]
#[doc(alias = "LOOP5STATE")]
pub type Loop5state = crate::Reg<loop5state::Loop5stateSpec>;
#[doc = "Loop N State Register"]
pub mod loop5state;
#[doc = "LOOP6STATE (rw) register accessor: Loop N State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop6state::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop6state::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop6state`] module"]
#[doc(alias = "LOOP6STATE")]
pub type Loop6state = crate::Reg<loop6state::Loop6stateSpec>;
#[doc = "Loop N State Register"]
pub mod loop6state;
#[doc = "LOOP7STATE (rw) register accessor: Loop N State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop7state::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop7state::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop7state`] module"]
#[doc(alias = "LOOP7STATE")]
pub type Loop7state = crate::Reg<loop7state::Loop7stateSpec>;
#[doc = "Loop N State Register"]
pub mod loop7state;
#[doc = "ALU0REGSTATE (rw) register accessor: ALU Register\n\nYou can [`read`](crate::Reg::read) this register and get [`alu0regstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`alu0regstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@alu0regstate`] module"]
#[doc(alias = "ALU0REGSTATE")]
pub type Alu0regstate = crate::Reg<alu0regstate::Alu0regstateSpec>;
#[doc = "ALU Register"]
pub mod alu0regstate;
#[doc = "ALU1REGSTATE (rw) register accessor: ALU Register\n\nYou can [`read`](crate::Reg::read) this register and get [`alu1regstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`alu1regstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@alu1regstate`] module"]
#[doc(alias = "ALU1REGSTATE")]
pub type Alu1regstate = crate::Reg<alu1regstate::Alu1regstateSpec>;
#[doc = "ALU Register"]
pub mod alu1regstate;
#[doc = "ALU2REGSTATE (rw) register accessor: ALU Register\n\nYou can [`read`](crate::Reg::read) this register and get [`alu2regstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`alu2regstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@alu2regstate`] module"]
#[doc(alias = "ALU2REGSTATE")]
pub type Alu2regstate = crate::Reg<alu2regstate::Alu2regstateSpec>;
#[doc = "ALU Register"]
pub mod alu2regstate;
#[doc = "ALU3REGSTATE (rw) register accessor: ALU Register\n\nYou can [`read`](crate::Reg::read) this register and get [`alu3regstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`alu3regstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@alu3regstate`] module"]
#[doc(alias = "ALU3REGSTATE")]
pub type Alu3regstate = crate::Reg<alu3regstate::Alu3regstateSpec>;
#[doc = "ALU Register"]
pub mod alu3regstate;
#[doc = "ALU4REGSTATE (rw) register accessor: ALU Register\n\nYou can [`read`](crate::Reg::read) this register and get [`alu4regstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`alu4regstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@alu4regstate`] module"]
#[doc(alias = "ALU4REGSTATE")]
pub type Alu4regstate = crate::Reg<alu4regstate::Alu4regstateSpec>;
#[doc = "ALU Register"]
pub mod alu4regstate;
#[doc = "ALU5REGSTATE (rw) register accessor: ALU Register\n\nYou can [`read`](crate::Reg::read) this register and get [`alu5regstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`alu5regstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@alu5regstate`] module"]
#[doc(alias = "ALU5REGSTATE")]
pub type Alu5regstate = crate::Reg<alu5regstate::Alu5regstateSpec>;
#[doc = "ALU Register"]
pub mod alu5regstate;
#[doc = "ALU6REGSTATE (rw) register accessor: ALU Register\n\nYou can [`read`](crate::Reg::read) this register and get [`alu6regstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`alu6regstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@alu6regstate`] module"]
#[doc(alias = "ALU6REGSTATE")]
pub type Alu6regstate = crate::Reg<alu6regstate::Alu6regstateSpec>;
#[doc = "ALU Register"]
pub mod alu6regstate;
#[doc = "ALU7REGSTATE (rw) register accessor: ALU Register\n\nYou can [`read`](crate::Reg::read) this register and get [`alu7regstate::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`alu7regstate::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@alu7regstate`] module"]
#[doc(alias = "ALU7REGSTATE")]
pub type Alu7regstate = crate::Reg<alu7regstate::Alu7regstateSpec>;
#[doc = "ALU Register"]
pub mod alu7regstate;
#[doc = "ARRAY0ADDRCFG (rw) register accessor: Array N Base Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array0addrcfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array0addrcfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array0addrcfg`] module"]
#[doc(alias = "ARRAY0ADDRCFG")]
pub type Array0addrcfg = crate::Reg<array0addrcfg::Array0addrcfgSpec>;
#[doc = "Array N Base Address Register"]
pub mod array0addrcfg;
#[doc = "ARRAY0DIM0CFG (rw) register accessor: Array N Dimenion 0 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array0dim0cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array0dim0cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array0dim0cfg`] module"]
#[doc(alias = "ARRAY0DIM0CFG")]
pub type Array0dim0cfg = crate::Reg<array0dim0cfg::Array0dim0cfgSpec>;
#[doc = "Array N Dimenion 0 Configuration"]
pub mod array0dim0cfg;
#[doc = "ARRAY0DIM1CFG (rw) register accessor: Array N Dimenion 1 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array0dim1cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array0dim1cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array0dim1cfg`] module"]
#[doc(alias = "ARRAY0DIM1CFG")]
pub type Array0dim1cfg = crate::Reg<array0dim1cfg::Array0dim1cfgSpec>;
#[doc = "Array N Dimenion 1 Configuration"]
pub mod array0dim1cfg;
#[doc = "ARRAY0DIM2CFG (rw) register accessor: Array N Dimenion 2 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array0dim2cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array0dim2cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array0dim2cfg`] module"]
#[doc(alias = "ARRAY0DIM2CFG")]
pub type Array0dim2cfg = crate::Reg<array0dim2cfg::Array0dim2cfgSpec>;
#[doc = "Array N Dimenion 2 Configuration"]
pub mod array0dim2cfg;
#[doc = "ARRAY1ADDRCFG (rw) register accessor: Array N Base Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array1addrcfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array1addrcfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array1addrcfg`] module"]
#[doc(alias = "ARRAY1ADDRCFG")]
pub type Array1addrcfg = crate::Reg<array1addrcfg::Array1addrcfgSpec>;
#[doc = "Array N Base Address Register"]
pub mod array1addrcfg;
#[doc = "ARRAY1DIM0CFG (rw) register accessor: Array N Dimenion 0 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array1dim0cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array1dim0cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array1dim0cfg`] module"]
#[doc(alias = "ARRAY1DIM0CFG")]
pub type Array1dim0cfg = crate::Reg<array1dim0cfg::Array1dim0cfgSpec>;
#[doc = "Array N Dimenion 0 Configuration"]
pub mod array1dim0cfg;
#[doc = "ARRAY1DIM1CFG (rw) register accessor: Array N Dimenion 1 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array1dim1cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array1dim1cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array1dim1cfg`] module"]
#[doc(alias = "ARRAY1DIM1CFG")]
pub type Array1dim1cfg = crate::Reg<array1dim1cfg::Array1dim1cfgSpec>;
#[doc = "Array N Dimenion 1 Configuration"]
pub mod array1dim1cfg;
#[doc = "ARRAY1DIM2CFG (rw) register accessor: Array N Dimenion 2 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array1dim2cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array1dim2cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array1dim2cfg`] module"]
#[doc(alias = "ARRAY1DIM2CFG")]
pub type Array1dim2cfg = crate::Reg<array1dim2cfg::Array1dim2cfgSpec>;
#[doc = "Array N Dimenion 2 Configuration"]
pub mod array1dim2cfg;
#[doc = "ARRAY2ADDRCFG (rw) register accessor: Array N Base Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array2addrcfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array2addrcfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array2addrcfg`] module"]
#[doc(alias = "ARRAY2ADDRCFG")]
pub type Array2addrcfg = crate::Reg<array2addrcfg::Array2addrcfgSpec>;
#[doc = "Array N Base Address Register"]
pub mod array2addrcfg;
#[doc = "ARRAY2DIM0CFG (rw) register accessor: Array N Dimenion 0 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array2dim0cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array2dim0cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array2dim0cfg`] module"]
#[doc(alias = "ARRAY2DIM0CFG")]
pub type Array2dim0cfg = crate::Reg<array2dim0cfg::Array2dim0cfgSpec>;
#[doc = "Array N Dimenion 0 Configuration"]
pub mod array2dim0cfg;
#[doc = "ARRAY2DIM1CFG (rw) register accessor: Array N Dimenion 1 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array2dim1cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array2dim1cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array2dim1cfg`] module"]
#[doc(alias = "ARRAY2DIM1CFG")]
pub type Array2dim1cfg = crate::Reg<array2dim1cfg::Array2dim1cfgSpec>;
#[doc = "Array N Dimenion 1 Configuration"]
pub mod array2dim1cfg;
#[doc = "ARRAY2DIM2CFG (rw) register accessor: Array N Dimenion 2 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array2dim2cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array2dim2cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array2dim2cfg`] module"]
#[doc(alias = "ARRAY2DIM2CFG")]
pub type Array2dim2cfg = crate::Reg<array2dim2cfg::Array2dim2cfgSpec>;
#[doc = "Array N Dimenion 2 Configuration"]
pub mod array2dim2cfg;
#[doc = "ARRAY3ADDRCFG (rw) register accessor: Array N Base Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array3addrcfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array3addrcfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array3addrcfg`] module"]
#[doc(alias = "ARRAY3ADDRCFG")]
pub type Array3addrcfg = crate::Reg<array3addrcfg::Array3addrcfgSpec>;
#[doc = "Array N Base Address Register"]
pub mod array3addrcfg;
#[doc = "ARRAY3DIM0CFG (rw) register accessor: Array N Dimenion 0 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array3dim0cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array3dim0cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array3dim0cfg`] module"]
#[doc(alias = "ARRAY3DIM0CFG")]
pub type Array3dim0cfg = crate::Reg<array3dim0cfg::Array3dim0cfgSpec>;
#[doc = "Array N Dimenion 0 Configuration"]
pub mod array3dim0cfg;
#[doc = "ARRAY3DIM1CFG (rw) register accessor: Array N Dimenion 1 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array3dim1cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array3dim1cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array3dim1cfg`] module"]
#[doc(alias = "ARRAY3DIM1CFG")]
pub type Array3dim1cfg = crate::Reg<array3dim1cfg::Array3dim1cfgSpec>;
#[doc = "Array N Dimenion 1 Configuration"]
pub mod array3dim1cfg;
#[doc = "ARRAY3DIM2CFG (rw) register accessor: Array N Dimenion 2 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array3dim2cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array3dim2cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array3dim2cfg`] module"]
#[doc(alias = "ARRAY3DIM2CFG")]
pub type Array3dim2cfg = crate::Reg<array3dim2cfg::Array3dim2cfgSpec>;
#[doc = "Array N Dimenion 2 Configuration"]
pub mod array3dim2cfg;
#[doc = "ARRAY4ADDRCFG (rw) register accessor: Array N Base Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array4addrcfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array4addrcfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array4addrcfg`] module"]
#[doc(alias = "ARRAY4ADDRCFG")]
pub type Array4addrcfg = crate::Reg<array4addrcfg::Array4addrcfgSpec>;
#[doc = "Array N Base Address Register"]
pub mod array4addrcfg;
#[doc = "ARRAY4DIM0CFG (rw) register accessor: Array N Dimenion 0 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array4dim0cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array4dim0cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array4dim0cfg`] module"]
#[doc(alias = "ARRAY4DIM0CFG")]
pub type Array4dim0cfg = crate::Reg<array4dim0cfg::Array4dim0cfgSpec>;
#[doc = "Array N Dimenion 0 Configuration"]
pub mod array4dim0cfg;
#[doc = "ARRAY4DIM1CFG (rw) register accessor: Array N Dimenion 1 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array4dim1cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array4dim1cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array4dim1cfg`] module"]
#[doc(alias = "ARRAY4DIM1CFG")]
pub type Array4dim1cfg = crate::Reg<array4dim1cfg::Array4dim1cfgSpec>;
#[doc = "Array N Dimenion 1 Configuration"]
pub mod array4dim1cfg;
#[doc = "ARRAY4DIM2CFG (rw) register accessor: Array N Dimenion 2 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array4dim2cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array4dim2cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@array4dim2cfg`] module"]
#[doc(alias = "ARRAY4DIM2CFG")]
pub type Array4dim2cfg = crate::Reg<array4dim2cfg::Array4dim2cfgSpec>;
#[doc = "Array N Dimenion 2 Configuration"]
pub mod array4dim2cfg;
#[doc = "LOOP0CFG (rw) register accessor: Loop N Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop0cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop0cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop0cfg`] module"]
#[doc(alias = "LOOP0CFG")]
pub type Loop0cfg = crate::Reg<loop0cfg::Loop0cfgSpec>;
#[doc = "Loop N Configuration Register"]
pub mod loop0cfg;
#[doc = "LOOP0RST (rw) register accessor: Loop N Reset Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop0rst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop0rst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop0rst`] module"]
#[doc(alias = "LOOP0RST")]
pub type Loop0rst = crate::Reg<loop0rst::Loop0rstSpec>;
#[doc = "Loop N Reset Configuration Register"]
pub mod loop0rst;
#[doc = "LOOP1CFG (rw) register accessor: Loop N Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop1cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop1cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop1cfg`] module"]
#[doc(alias = "LOOP1CFG")]
pub type Loop1cfg = crate::Reg<loop1cfg::Loop1cfgSpec>;
#[doc = "Loop N Configuration Register"]
pub mod loop1cfg;
#[doc = "LOOP1RST (rw) register accessor: Loop N Reset Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop1rst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop1rst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop1rst`] module"]
#[doc(alias = "LOOP1RST")]
pub type Loop1rst = crate::Reg<loop1rst::Loop1rstSpec>;
#[doc = "Loop N Reset Configuration Register"]
pub mod loop1rst;
#[doc = "LOOP2CFG (rw) register accessor: Loop N Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop2cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop2cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop2cfg`] module"]
#[doc(alias = "LOOP2CFG")]
pub type Loop2cfg = crate::Reg<loop2cfg::Loop2cfgSpec>;
#[doc = "Loop N Configuration Register"]
pub mod loop2cfg;
#[doc = "LOOP2RST (rw) register accessor: Loop N Reset Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop2rst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop2rst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop2rst`] module"]
#[doc(alias = "LOOP2RST")]
pub type Loop2rst = crate::Reg<loop2rst::Loop2rstSpec>;
#[doc = "Loop N Reset Configuration Register"]
pub mod loop2rst;
#[doc = "LOOP3CFG (rw) register accessor: Loop N Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop3cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop3cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop3cfg`] module"]
#[doc(alias = "LOOP3CFG")]
pub type Loop3cfg = crate::Reg<loop3cfg::Loop3cfgSpec>;
#[doc = "Loop N Configuration Register"]
pub mod loop3cfg;
#[doc = "LOOP3RST (rw) register accessor: Loop N Reset Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop3rst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop3rst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop3rst`] module"]
#[doc(alias = "LOOP3RST")]
pub type Loop3rst = crate::Reg<loop3rst::Loop3rstSpec>;
#[doc = "Loop N Reset Configuration Register"]
pub mod loop3rst;
#[doc = "LOOP4CFG (rw) register accessor: Loop N Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop4cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop4cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop4cfg`] module"]
#[doc(alias = "LOOP4CFG")]
pub type Loop4cfg = crate::Reg<loop4cfg::Loop4cfgSpec>;
#[doc = "Loop N Configuration Register"]
pub mod loop4cfg;
#[doc = "LOOP4RST (rw) register accessor: Loop N Reset Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop4rst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop4rst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop4rst`] module"]
#[doc(alias = "LOOP4RST")]
pub type Loop4rst = crate::Reg<loop4rst::Loop4rstSpec>;
#[doc = "Loop N Reset Configuration Register"]
pub mod loop4rst;
#[doc = "LOOP5CFG (rw) register accessor: Loop N Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop5cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop5cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop5cfg`] module"]
#[doc(alias = "LOOP5CFG")]
pub type Loop5cfg = crate::Reg<loop5cfg::Loop5cfgSpec>;
#[doc = "Loop N Configuration Register"]
pub mod loop5cfg;
#[doc = "LOOP5RST (rw) register accessor: Loop N Reset Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop5rst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop5rst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop5rst`] module"]
#[doc(alias = "LOOP5RST")]
pub type Loop5rst = crate::Reg<loop5rst::Loop5rstSpec>;
#[doc = "Loop N Reset Configuration Register"]
pub mod loop5rst;
#[doc = "LOOP6CFG (rw) register accessor: Loop N Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop6cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop6cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop6cfg`] module"]
#[doc(alias = "LOOP6CFG")]
pub type Loop6cfg = crate::Reg<loop6cfg::Loop6cfgSpec>;
#[doc = "Loop N Configuration Register"]
pub mod loop6cfg;
#[doc = "LOOP6RST (rw) register accessor: Loop N Reset Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop6rst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop6rst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop6rst`] module"]
#[doc(alias = "LOOP6RST")]
pub type Loop6rst = crate::Reg<loop6rst::Loop6rstSpec>;
#[doc = "Loop N Reset Configuration Register"]
pub mod loop6rst;
#[doc = "LOOP7CFG (rw) register accessor: Loop N Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop7cfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop7cfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop7cfg`] module"]
#[doc(alias = "LOOP7CFG")]
pub type Loop7cfg = crate::Reg<loop7cfg::Loop7cfgSpec>;
#[doc = "Loop N Configuration Register"]
pub mod loop7cfg;
#[doc = "LOOP7RST (rw) register accessor: Loop N Reset Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop7rst::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop7rst::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@loop7rst`] module"]
#[doc(alias = "LOOP7RST")]
pub type Loop7rst = crate::Reg<loop7rst::Loop7rstSpec>;
#[doc = "Loop N Reset Configuration Register"]
pub mod loop7rst;
#[doc = "INSTR0CFG0 (rw) register accessor: Instruction N Word 0\n\nYou can [`read`](crate::Reg::read) this register and get [`instr0cfg0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr0cfg0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr0cfg0`] module"]
#[doc(alias = "INSTR0CFG0")]
pub type Instr0cfg0 = crate::Reg<instr0cfg0::Instr0cfg0Spec>;
#[doc = "Instruction N Word 0"]
pub mod instr0cfg0;
#[doc = "INSTR0CFG1 (rw) register accessor: Instruction N word 1\n\nYou can [`read`](crate::Reg::read) this register and get [`instr0cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr0cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr0cfg1`] module"]
#[doc(alias = "INSTR0CFG1")]
pub type Instr0cfg1 = crate::Reg<instr0cfg1::Instr0cfg1Spec>;
#[doc = "Instruction N word 1"]
pub mod instr0cfg1;
#[doc = "INSTR0CFG2 (rw) register accessor: Instruction N word 2\n\nYou can [`read`](crate::Reg::read) this register and get [`instr0cfg2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr0cfg2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr0cfg2`] module"]
#[doc(alias = "INSTR0CFG2")]
pub type Instr0cfg2 = crate::Reg<instr0cfg2::Instr0cfg2Spec>;
#[doc = "Instruction N word 2"]
pub mod instr0cfg2;
#[doc = "INSTR1CFG0 (rw) register accessor: Instruction N Word 0\n\nYou can [`read`](crate::Reg::read) this register and get [`instr1cfg0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr1cfg0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr1cfg0`] module"]
#[doc(alias = "INSTR1CFG0")]
pub type Instr1cfg0 = crate::Reg<instr1cfg0::Instr1cfg0Spec>;
#[doc = "Instruction N Word 0"]
pub mod instr1cfg0;
#[doc = "INSTR1CFG1 (rw) register accessor: Instruction N word 1\n\nYou can [`read`](crate::Reg::read) this register and get [`instr1cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr1cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr1cfg1`] module"]
#[doc(alias = "INSTR1CFG1")]
pub type Instr1cfg1 = crate::Reg<instr1cfg1::Instr1cfg1Spec>;
#[doc = "Instruction N word 1"]
pub mod instr1cfg1;
#[doc = "INSTR1CFG2 (rw) register accessor: Instruction N word 2\n\nYou can [`read`](crate::Reg::read) this register and get [`instr1cfg2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr1cfg2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr1cfg2`] module"]
#[doc(alias = "INSTR1CFG2")]
pub type Instr1cfg2 = crate::Reg<instr1cfg2::Instr1cfg2Spec>;
#[doc = "Instruction N word 2"]
pub mod instr1cfg2;
#[doc = "INSTR2CFG0 (rw) register accessor: Instruction N Word 0\n\nYou can [`read`](crate::Reg::read) this register and get [`instr2cfg0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr2cfg0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr2cfg0`] module"]
#[doc(alias = "INSTR2CFG0")]
pub type Instr2cfg0 = crate::Reg<instr2cfg0::Instr2cfg0Spec>;
#[doc = "Instruction N Word 0"]
pub mod instr2cfg0;
#[doc = "INSTR2CFG1 (rw) register accessor: Instruction N word 1\n\nYou can [`read`](crate::Reg::read) this register and get [`instr2cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr2cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr2cfg1`] module"]
#[doc(alias = "INSTR2CFG1")]
pub type Instr2cfg1 = crate::Reg<instr2cfg1::Instr2cfg1Spec>;
#[doc = "Instruction N word 1"]
pub mod instr2cfg1;
#[doc = "INSTR2CFG2 (rw) register accessor: Instruction N word 2\n\nYou can [`read`](crate::Reg::read) this register and get [`instr2cfg2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr2cfg2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr2cfg2`] module"]
#[doc(alias = "INSTR2CFG2")]
pub type Instr2cfg2 = crate::Reg<instr2cfg2::Instr2cfg2Spec>;
#[doc = "Instruction N word 2"]
pub mod instr2cfg2;
#[doc = "INSTR3CFG0 (rw) register accessor: Instruction N Word 0\n\nYou can [`read`](crate::Reg::read) this register and get [`instr3cfg0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr3cfg0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr3cfg0`] module"]
#[doc(alias = "INSTR3CFG0")]
pub type Instr3cfg0 = crate::Reg<instr3cfg0::Instr3cfg0Spec>;
#[doc = "Instruction N Word 0"]
pub mod instr3cfg0;
#[doc = "INSTR3CFG1 (rw) register accessor: Instruction N word 1\n\nYou can [`read`](crate::Reg::read) this register and get [`instr3cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr3cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr3cfg1`] module"]
#[doc(alias = "INSTR3CFG1")]
pub type Instr3cfg1 = crate::Reg<instr3cfg1::Instr3cfg1Spec>;
#[doc = "Instruction N word 1"]
pub mod instr3cfg1;
#[doc = "INSTR3CFG2 (rw) register accessor: Instruction N word 2\n\nYou can [`read`](crate::Reg::read) this register and get [`instr3cfg2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr3cfg2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr3cfg2`] module"]
#[doc(alias = "INSTR3CFG2")]
pub type Instr3cfg2 = crate::Reg<instr3cfg2::Instr3cfg2Spec>;
#[doc = "Instruction N word 2"]
pub mod instr3cfg2;
#[doc = "INSTR4CFG0 (rw) register accessor: Instruction N Word 0\n\nYou can [`read`](crate::Reg::read) this register and get [`instr4cfg0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr4cfg0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr4cfg0`] module"]
#[doc(alias = "INSTR4CFG0")]
pub type Instr4cfg0 = crate::Reg<instr4cfg0::Instr4cfg0Spec>;
#[doc = "Instruction N Word 0"]
pub mod instr4cfg0;
#[doc = "INSTR4CFG1 (rw) register accessor: Instruction N word 1\n\nYou can [`read`](crate::Reg::read) this register and get [`instr4cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr4cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr4cfg1`] module"]
#[doc(alias = "INSTR4CFG1")]
pub type Instr4cfg1 = crate::Reg<instr4cfg1::Instr4cfg1Spec>;
#[doc = "Instruction N word 1"]
pub mod instr4cfg1;
#[doc = "INSTR4CFG2 (rw) register accessor: Instruction N word 2\n\nYou can [`read`](crate::Reg::read) this register and get [`instr4cfg2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr4cfg2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr4cfg2`] module"]
#[doc(alias = "INSTR4CFG2")]
pub type Instr4cfg2 = crate::Reg<instr4cfg2::Instr4cfg2Spec>;
#[doc = "Instruction N word 2"]
pub mod instr4cfg2;
#[doc = "INSTR5CFG0 (rw) register accessor: Instruction N Word 0\n\nYou can [`read`](crate::Reg::read) this register and get [`instr5cfg0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr5cfg0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr5cfg0`] module"]
#[doc(alias = "INSTR5CFG0")]
pub type Instr5cfg0 = crate::Reg<instr5cfg0::Instr5cfg0Spec>;
#[doc = "Instruction N Word 0"]
pub mod instr5cfg0;
#[doc = "INSTR5CFG1 (rw) register accessor: Instruction N word 1\n\nYou can [`read`](crate::Reg::read) this register and get [`instr5cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr5cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr5cfg1`] module"]
#[doc(alias = "INSTR5CFG1")]
pub type Instr5cfg1 = crate::Reg<instr5cfg1::Instr5cfg1Spec>;
#[doc = "Instruction N word 1"]
pub mod instr5cfg1;
#[doc = "INSTR5CFG2 (rw) register accessor: Instruction N word 2\n\nYou can [`read`](crate::Reg::read) this register and get [`instr5cfg2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr5cfg2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr5cfg2`] module"]
#[doc(alias = "INSTR5CFG2")]
pub type Instr5cfg2 = crate::Reg<instr5cfg2::Instr5cfg2Spec>;
#[doc = "Instruction N word 2"]
pub mod instr5cfg2;
#[doc = "INSTR6CFG0 (rw) register accessor: Instruction N Word 0\n\nYou can [`read`](crate::Reg::read) this register and get [`instr6cfg0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr6cfg0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr6cfg0`] module"]
#[doc(alias = "INSTR6CFG0")]
pub type Instr6cfg0 = crate::Reg<instr6cfg0::Instr6cfg0Spec>;
#[doc = "Instruction N Word 0"]
pub mod instr6cfg0;
#[doc = "INSTR6CFG1 (rw) register accessor: Instruction N word 1\n\nYou can [`read`](crate::Reg::read) this register and get [`instr6cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr6cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr6cfg1`] module"]
#[doc(alias = "INSTR6CFG1")]
pub type Instr6cfg1 = crate::Reg<instr6cfg1::Instr6cfg1Spec>;
#[doc = "Instruction N word 1"]
pub mod instr6cfg1;
#[doc = "INSTR6CFG2 (rw) register accessor: Instruction N word 2\n\nYou can [`read`](crate::Reg::read) this register and get [`instr6cfg2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr6cfg2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr6cfg2`] module"]
#[doc(alias = "INSTR6CFG2")]
pub type Instr6cfg2 = crate::Reg<instr6cfg2::Instr6cfg2Spec>;
#[doc = "Instruction N word 2"]
pub mod instr6cfg2;
#[doc = "INSTR7CFG0 (rw) register accessor: Instruction N Word 0\n\nYou can [`read`](crate::Reg::read) this register and get [`instr7cfg0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr7cfg0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr7cfg0`] module"]
#[doc(alias = "INSTR7CFG0")]
pub type Instr7cfg0 = crate::Reg<instr7cfg0::Instr7cfg0Spec>;
#[doc = "Instruction N Word 0"]
pub mod instr7cfg0;
#[doc = "INSTR7CFG1 (rw) register accessor: Instruction N word 1\n\nYou can [`read`](crate::Reg::read) this register and get [`instr7cfg1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr7cfg1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr7cfg1`] module"]
#[doc(alias = "INSTR7CFG1")]
pub type Instr7cfg1 = crate::Reg<instr7cfg1::Instr7cfg1Spec>;
#[doc = "Instruction N word 1"]
pub mod instr7cfg1;
#[doc = "INSTR7CFG2 (rw) register accessor: Instruction N word 2\n\nYou can [`read`](crate::Reg::read) this register and get [`instr7cfg2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr7cfg2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@instr7cfg2`] module"]
#[doc(alias = "INSTR7CFG2")]
pub type Instr7cfg2 = crate::Reg<instr7cfg2::Instr7cfg2Spec>;
#[doc = "Instruction N word 2"]
pub mod instr7cfg2;
#[doc = "CMD (w) register accessor: Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmd::W`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@cmd`] module"]
#[doc(alias = "CMD")]
pub type Cmd = crate::Reg<cmd::CmdSpec>;
#[doc = "Command Register"]
pub mod cmd;
#[doc = "DEBUGEN (rw) register accessor: Debug Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`debugen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debugen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@debugen`] module"]
#[doc(alias = "DEBUGEN")]
pub type Debugen = crate::Reg<debugen::DebugenSpec>;
#[doc = "Debug Control Register"]
pub mod debugen;
#[doc = "DEBUGSTEPCNT (rw) register accessor: No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`debugstepcnt::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debugstepcnt::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@debugstepcnt`] module"]
#[doc(alias = "DEBUGSTEPCNT")]
pub type Debugstepcnt = crate::Reg<debugstepcnt::DebugstepcntSpec>;
#[doc = "No Description"]
pub mod debugstepcnt;
