//! Lowering rules for EVM.

use crate::ir::Inst as IRInst;
use crate::isa::evm::EvmBackend;
use crate::isa::evm::inst::Inst;
use crate::machinst::lower::*;
use crate::machinst::*;

pub(super) mod isle;

type MInst = Inst;

impl LowerBackend for EvmBackend {
    type MInst = MInst;

    fn lower(&self, ctx: &mut Lower<MInst>, ir_inst: IRInst) -> Option<InstOutput> {
        isle::lower(ctx, self, ir_inst)
    }

    fn lower_branch(
        &self,
        ctx: &mut Lower<MInst>,
        ir_inst: IRInst,
        targets: &[MachLabel],
    ) -> Option<()> {
        isle::lower_branch(ctx, self, ir_inst, targets)
    }

    fn maybe_pinned_reg(&self) -> Option<Reg> {
        None
    }

    type FactFlowState = ();
}
