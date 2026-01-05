//! ISLE integration glue code for EVM lowering.

pub mod generated_code;
use generated_code::MInst;

use crate::ir::condcodes::{FloatCC, IntCC};
use crate::ir::Inst;
use crate::isa::evm::EvmBackend;
use crate::isa::evm::inst::ImmBytes;
use crate::machinst::Reg;
use crate::machinst::{CallInfo, MachInst, isle::*};
use crate::machinst::{VCodeConstant, VCodeConstantData};
use crate::{
    ir::{
        BlockCall, ExternalName, InstructionData, MemFlags, Opcode, TrapCode, Value, ValueList,
        immediates::*, types::*,
    },
    machinst::{ArgPair, CallArgList, CallRetList, InstOutput, MachLabel},
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use regalloc2::PReg;

#[allow(dead_code)]
type BoxCallInfo = Box<CallInfo<ExternalName>>;
#[allow(dead_code)]
type BoxCallIndInfo = Box<CallInfo<Reg>>;
#[allow(dead_code)]
type BoxExternalName = Box<ExternalName>;
#[allow(dead_code)]
type VecMachLabel = Vec<MachLabel>;
#[allow(dead_code)]
type VecArgPair = Vec<ArgPair>;

pub(crate) struct EvmIsleContext<'a, 'b, I, B>
where
    I: VCodeInst,
    B: LowerBackend,
{
    pub lower_ctx: &'a mut Lower<'b, I>,
    #[allow(dead_code)]
    pub backend: &'a B,
}

impl<'a, 'b> EvmIsleContext<'a, 'b, MInst, EvmBackend> {
    fn new(lower_ctx: &'a mut Lower<'b, MInst>, backend: &'a EvmBackend) -> Self {
        Self { lower_ctx, backend }
    }

    fn dfg(&self) -> &crate::ir::DataFlowGraph {
        &self.lower_ctx.f.dfg
    }
}

impl generated_code::Context for EvmIsleContext<'_, '_, MInst, EvmBackend> {
    isle_lower_prelude_methods!();

    fn emit(&mut self, inst: &MInst) -> Unit {
        self.lower_ctx.emit(inst.clone());
    }

    fn imm(&mut self, _ty: Type, val: u64) -> InstOutput {
        let mut bytes = [0u8; 32];
        bytes[24..].copy_from_slice(&val.to_be_bytes());
        self.lower_ctx.emit(MInst::Push { imm: bytes });
        InstOutput::new()
    }

    fn evm_binary_op(&mut self, op: &MInst, _ty: Type, _x: Value, _y: Value) -> InstOutput {
        self.lower_ctx.emit(op.clone());
        InstOutput::new()
    }

    fn evm_unary_op(&mut self, op: &MInst, _ty: Type, _x: Value) -> InstOutput {
        self.lower_ctx.emit(op.clone());
        InstOutput::new()
    }

    fn evm_trap(&mut self, code: &TrapCode) -> InstOutput {
        self.lower_ctx.emit(MInst::Trap { code: *code });
        InstOutput::new()
    }
}

pub fn lower(
    ctx: &mut Lower<MInst>,
    backend: &EvmBackend,
    ir_inst: Inst,
) -> Option<InstOutput> {
    let mut isle_ctx = EvmIsleContext::new(ctx, backend);
    generated_code::constructor_lower(&mut isle_ctx, ir_inst)
}

pub fn lower_branch(
    ctx: &mut Lower<MInst>,
    backend: &EvmBackend,
    ir_inst: Inst,
    targets: &[MachLabel],
) -> Option<()> {
    let mut isle_ctx = EvmIsleContext::new(ctx, backend);
    generated_code::constructor_lower_branch(&mut isle_ctx, ir_inst, targets)
}
