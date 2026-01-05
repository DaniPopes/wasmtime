//! EVM ABI implementation.
//!
//! The EVM is a stack-based machine and doesn't have a traditional ABI with
//! calling conventions. This module provides a minimal implementation to
//! satisfy Cranelift's ABI requirements.

use crate::ir;
use crate::ir::types::*;
use crate::ir::Signature;
use crate::isa;
use crate::isa::CallConv;
use crate::isa::evm::inst::*;
use crate::isa::evm::settings::Flags as EvmFlags;
use crate::machinst::*;
use crate::settings;
use crate::CodegenResult;

use alloc::vec::Vec;
use regalloc2::{MachineEnv, PReg, PRegSet};
use smallvec::{SmallVec, smallvec};
use std::sync::OnceLock;

pub(crate) type EvmCallee = Callee<EvmMachineDeps>;

pub struct EvmMachineDeps;

impl IsaFlags for EvmFlags {}

impl ABIMachineSpec for EvmMachineDeps {
    type I = Inst;
    type F = EvmFlags;

    const STACK_ARG_RET_SIZE_LIMIT: u32 = 128 * 1024 * 1024;

    fn word_bits() -> u32 {
        256
    }

    fn stack_align(_call_conv: isa::CallConv) -> u32 {
        32
    }

    fn compute_arg_locs(
        _call_conv: isa::CallConv,
        _flags: &settings::Flags,
        params: &[ir::AbiParam],
        _args_or_rets: ArgsOrRets,
        _add_ret_area_ptr: bool,
        mut args: ArgsAccumulator,
    ) -> CodegenResult<(u32, Option<usize>)> {
        for (i, param) in params.iter().enumerate() {
            args.push(ABIArg::reg(
                regs::gpr_preg(i as u8).into(),
                param.value_type,
                param.extension,
                param.purpose,
            ));
        }
        Ok((0, None))
    }

    fn gen_load_stack(_mem: StackAMode, _into_reg: Writable<Reg>, _ty: Type) -> Self::I {
        Inst::MLoad
    }

    fn gen_store_stack(_mem: StackAMode, _from_reg: Reg, _ty: Type) -> Self::I {
        Inst::MStore
    }

    fn gen_move(_to_reg: Writable<Reg>, _from_reg: Reg, _ty: Type) -> Self::I {
        Inst::Nop
    }

    fn gen_extend(
        _to_reg: Writable<Reg>,
        _from_reg: Reg,
        _signed: bool,
        _from_bits: u8,
        _to_bits: u8,
    ) -> Self::I {
        Inst::Nop
    }

    fn gen_args(_args: Vec<ArgPair>) -> Self::I {
        Inst::Nop
    }

    fn gen_rets(_rets: Vec<RetPair>) -> Self::I {
        Inst::Return
    }

    fn gen_add_imm(
        _call_conv: isa::CallConv,
        _into_reg: Writable<Reg>,
        _from_reg: Reg,
        _imm: u32,
    ) -> SmallInstVec<Self::I> {
        smallvec![Inst::Add]
    }

    fn gen_stack_lower_bound_trap(_limit_reg: Reg) -> SmallInstVec<Self::I> {
        smallvec![]
    }

    fn gen_get_stack_addr(_mem: StackAMode, _into_reg: Writable<Reg>) -> Self::I {
        Inst::Nop
    }

    fn get_stacklimit_reg(_call_conv: isa::CallConv) -> Reg {
        regs::gpr(0)
    }

    fn gen_load_base_offset(
        _into_reg: Writable<Reg>,
        _base: Reg,
        _offset: i32,
        _ty: Type,
    ) -> Self::I {
        Inst::MLoad
    }

    fn gen_store_base_offset(_base: Reg, _offset: i32, _from_reg: Reg, _ty: Type) -> Self::I {
        Inst::MStore
    }

    fn gen_sp_reg_adjust(_amount: i32) -> SmallInstVec<Self::I> {
        smallvec![]
    }

    fn compute_frame_layout(
        _call_conv: CallConv,
        _flags: &settings::Flags,
        _sig: &Signature,
        _regs: &[Writable<RealReg>],
        _function_calls: FunctionCalls,
        _incoming_args_size: u32,
        _tail_args_size: u32,
        _stackslots_size: u32,
        _fixed_frame_storage_size: u32,
        _outgoing_args_size: u32,
    ) -> FrameLayout {
        FrameLayout {
            word_bytes: 32,
            incoming_args_size: 0,
            tail_args_size: 0,
            setup_area_size: 0,
            clobber_size: 0,
            fixed_frame_storage_size: 0,
            stackslots_size: 0,
            outgoing_args_size: 0,
            clobbered_callee_saves: Vec::new(),
            function_calls: FunctionCalls::None,
        }
    }

    fn gen_prologue_frame_setup(
        _call_conv: isa::CallConv,
        _flags: &settings::Flags,
        _isa_flags: &Self::F,
        _frame_layout: &FrameLayout,
    ) -> SmallInstVec<Self::I> {
        smallvec![]
    }

    fn gen_epilogue_frame_restore(
        _call_conv: isa::CallConv,
        _flags: &settings::Flags,
        _isa_flags: &Self::F,
        _frame_layout: &FrameLayout,
    ) -> SmallInstVec<Self::I> {
        smallvec![]
    }

    fn gen_return(
        _call_conv: isa::CallConv,
        _isa_flags: &Self::F,
        _frame_layout: &FrameLayout,
    ) -> SmallInstVec<Self::I> {
        smallvec![Inst::Return]
    }

    fn gen_probestack(_insts: &mut SmallInstVec<Self::I>, _frame_size: u32) {}

    fn gen_inline_probestack(
        _insts: &mut SmallInstVec<Self::I>,
        _call_conv: isa::CallConv,
        _frame_size: u32,
        _guard_size: u32,
    ) {
    }

    fn gen_clobber_save(
        _call_conv: CallConv,
        _flags: &settings::Flags,
        _frame_layout: &FrameLayout,
    ) -> SmallVec<[Self::I; 16]> {
        SmallVec::new()
    }

    fn gen_clobber_restore(
        _call_conv: CallConv,
        _flags: &settings::Flags,
        _frame_layout: &FrameLayout,
    ) -> SmallVec<[Self::I; 16]> {
        SmallVec::new()
    }

    fn gen_memcpy<F: FnMut(Type) -> Writable<Reg>>(
        _call_conv: isa::CallConv,
        _dst: Reg,
        _src: Reg,
        _size: usize,
        _alloc_tmp: F,
    ) -> SmallVec<[Self::I; 8]> {
        smallvec![Inst::MCopy]
    }

    fn get_number_of_spillslots_for_value(
        _rc: RegClass,
        _target_vector_bytes: u32,
        _isa_flags: &Self::F,
    ) -> u32 {
        1
    }

    fn get_machine_env(_flags: &settings::Flags, _call_conv: CallConv) -> &MachineEnv {
        static MACHINE_ENV: OnceLock<MachineEnv> = OnceLock::new();
        MACHINE_ENV.get_or_init(|| {
            let preferred_regs: Vec<PReg> = (0..16).map(|i| regs::gpr_preg(i)).collect();
            let non_preferred_regs: Vec<PReg> = vec![];

            MachineEnv {
                preferred_regs_by_class: [preferred_regs, vec![], vec![]],
                non_preferred_regs_by_class: [non_preferred_regs, vec![], vec![]],
                fixed_stack_slots: vec![],
                scratch_by_class: [None, None, None],
            }
        })
    }

    fn get_regs_clobbered_by_call(_call_conv: CallConv, _is_exception: bool) -> PRegSet {
        PRegSet::empty()
    }

    fn get_ext_mode(
        _call_conv: isa::CallConv,
        specified: ir::ArgumentExtension,
    ) -> ir::ArgumentExtension {
        specified
    }

    fn retval_temp_reg(_call_conv_of_callee: isa::CallConv) -> Writable<Reg> {
        Writable::from_reg(regs::gpr(0))
    }
}
