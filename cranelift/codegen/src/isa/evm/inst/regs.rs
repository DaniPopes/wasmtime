//! EVM register definitions.
//!
//! The EVM is a pure stack machine, but Cranelift requires registers for its
//! internal representation. We define virtual registers that will be mapped
//! to stack operations during code emission.
//!
//! EVM operates on 256-bit words, so we define a set of virtual registers
//! that represent stack slots.

use crate::machinst::{Reg, Writable};
use regalloc2::{PReg, RegClass, VReg};

pub const EVM_STACK_DEPTH: usize = 16;

pub fn gpr(index: u8) -> Reg {
    let preg = gpr_preg(index);
    Reg::from(VReg::new(preg.index(), preg.class()))
}

pub fn writable_gpr(index: u8) -> Writable<Reg> {
    Writable::from_reg(gpr(index))
}

pub fn gpr_preg(index: u8) -> PReg {
    debug_assert!(index < EVM_STACK_DEPTH as u8);
    PReg::new(index as usize, RegClass::Int)
}

pub fn reg_name(reg: Reg) -> &'static str {
    match reg.to_real_reg() {
        Some(real) => match real.hw_enc() {
            0 => "s0",
            1 => "s1",
            2 => "s2",
            3 => "s3",
            4 => "s4",
            5 => "s5",
            6 => "s6",
            7 => "s7",
            8 => "s8",
            9 => "s9",
            10 => "s10",
            11 => "s11",
            12 => "s12",
            13 => "s13",
            14 => "s14",
            15 => "s15",
            _ => "s?",
        },
        None => "v?",
    }
}

pub fn show_reg(reg: Reg) -> alloc::string::String {
    use alloc::string::ToString;
    reg_name(reg).to_string()
}
