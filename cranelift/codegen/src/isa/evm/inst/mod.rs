//! EVM instruction definitions.
//!
//! The EVM is a stack-based virtual machine with 256-bit words.
//! Instructions operate on a stack of up to 1024 elements.

use crate::binemit::{Addend, CodeOffset, Reloc};
use crate::ir::types::I64;
use crate::ir::Type;
use crate::isa::FunctionAlignment;
use crate::machinst::*;
use crate::settings;

use alloc::string::String;
use alloc::vec::Vec;
use regalloc2::RegClass;

pub mod emit;
pub mod regs;

pub use emit::*;
pub use regs::*;

use crate::isa::evm::abi::EvmMachineDeps;

pub use crate::isa::evm::lower::isle::generated_code::MInst as Inst;


pub(crate) type ImmBytes = [u8; 32];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelUse {
    Jump,
}

impl MachInstLabelUse for LabelUse {
    const ALIGN: CodeOffset = 1;

    fn max_pos_range(self) -> CodeOffset {
        u32::MAX
    }

    fn max_neg_range(self) -> CodeOffset {
        0
    }

    fn patch_size(self) -> CodeOffset {
        0
    }

    fn patch(self, _buffer: &mut [u8], _use_offset: CodeOffset, _label_offset: CodeOffset) {}

    fn supports_veneer(self) -> bool {
        false
    }

    fn veneer_size(self) -> CodeOffset {
        0
    }

    fn worst_case_veneer_size() -> CodeOffset {
        0
    }

    fn generate_veneer(
        self,
        _buffer: &mut [u8],
        _veneer_offset: CodeOffset,
    ) -> (CodeOffset, LabelUse) {
        (0, LabelUse::Jump)
    }

    fn from_reloc(_reloc: Reloc, _addend: Addend) -> Option<LabelUse> {
        None
    }
}

impl Inst {
    pub fn print_with_state(&self, _state: &mut EmitState) -> String {
        use core::fmt::Write;
        let mut s = String::new();
        let _ = match self {
            Inst::Nop => write!(s, "nop"),
            Inst::Stop => write!(s, "stop"),
            Inst::Push { imm } => {
                let leading_zeros = imm.iter().take_while(|&&b| b == 0).count();
                let significant = &imm[leading_zeros..];
                if significant.is_empty() {
                    write!(s, "push0")
                } else {
                    let _ = write!(s, "push{} 0x", significant.len());
                    for b in significant {
                        let _ = write!(s, "{:02x}", b);
                    }
                    Ok(())
                }
            }
            Inst::Pop => write!(s, "pop"),
            Inst::Dup { depth } => write!(s, "dup{}", depth),
            Inst::Swap { depth } => write!(s, "swap{}", depth),
            Inst::Add => write!(s, "add"),
            Inst::Mul => write!(s, "mul"),
            Inst::Sub => write!(s, "sub"),
            Inst::Div => write!(s, "div"),
            Inst::SDiv => write!(s, "sdiv"),
            Inst::Mod => write!(s, "mod"),
            Inst::SMod => write!(s, "smod"),
            Inst::AddMod => write!(s, "addmod"),
            Inst::MulMod => write!(s, "mulmod"),
            Inst::Exp => write!(s, "exp"),
            Inst::SignExtend => write!(s, "signextend"),
            Inst::Lt => write!(s, "lt"),
            Inst::Gt => write!(s, "gt"),
            Inst::SLt => write!(s, "slt"),
            Inst::SGt => write!(s, "sgt"),
            Inst::Eq => write!(s, "eq"),
            Inst::IsZero => write!(s, "iszero"),
            Inst::And => write!(s, "and"),
            Inst::Or => write!(s, "or"),
            Inst::Xor => write!(s, "xor"),
            Inst::Not => write!(s, "not"),
            Inst::Byte => write!(s, "byte"),
            Inst::Shl => write!(s, "shl"),
            Inst::Shr => write!(s, "shr"),
            Inst::Sar => write!(s, "sar"),
            Inst::Keccak256 => write!(s, "keccak256"),
            Inst::MLoad => write!(s, "mload"),
            Inst::MStore => write!(s, "mstore"),
            Inst::MStore8 => write!(s, "mstore8"),
            Inst::SLoad => write!(s, "sload"),
            Inst::SStore => write!(s, "sstore"),
            Inst::Jump { target } => write!(s, "jump {:?}", target),
            Inst::JumpI { target } => write!(s, "jumpi {:?}", target),
            Inst::JumpDest => write!(s, "jumpdest"),
            Inst::Pc => write!(s, "pc"),
            Inst::MSize => write!(s, "msize"),
            Inst::Gas => write!(s, "gas"),
            Inst::Return => write!(s, "return"),
            Inst::Revert => write!(s, "revert"),
            Inst::Invalid => write!(s, "invalid"),
            Inst::SelfDestruct => write!(s, "selfdestruct"),
            Inst::Address => write!(s, "address"),
            Inst::Balance => write!(s, "balance"),
            Inst::Origin => write!(s, "origin"),
            Inst::Caller => write!(s, "caller"),
            Inst::CallValue => write!(s, "callvalue"),
            Inst::CallDataLoad => write!(s, "calldataload"),
            Inst::CallDataSize => write!(s, "calldatasize"),
            Inst::CallDataCopy => write!(s, "calldatacopy"),
            Inst::CodeSize => write!(s, "codesize"),
            Inst::CodeCopy => write!(s, "codecopy"),
            Inst::GasPrice => write!(s, "gasprice"),
            Inst::ExtCodeSize => write!(s, "extcodesize"),
            Inst::ExtCodeCopy => write!(s, "extcodecopy"),
            Inst::ReturnDataSize => write!(s, "returndatasize"),
            Inst::ReturnDataCopy => write!(s, "returndatacopy"),
            Inst::ExtCodeHash => write!(s, "extcodehash"),
            Inst::BlockHash => write!(s, "blockhash"),
            Inst::Coinbase => write!(s, "coinbase"),
            Inst::Timestamp => write!(s, "timestamp"),
            Inst::Number => write!(s, "number"),
            Inst::PrevRandao => write!(s, "prevrandao"),
            Inst::GasLimit => write!(s, "gaslimit"),
            Inst::ChainId => write!(s, "chainid"),
            Inst::SelfBalance => write!(s, "selfbalance"),
            Inst::BaseFee => write!(s, "basefee"),
            Inst::Log { topic_count } => write!(s, "log{}", topic_count),
            Inst::Call => write!(s, "call"),
            Inst::CallCode => write!(s, "callcode"),
            Inst::DelegateCall => write!(s, "delegatecall"),
            Inst::StaticCall => write!(s, "staticcall"),
            Inst::Create => write!(s, "create"),
            Inst::Create2 => write!(s, "create2"),
            Inst::TLoad => write!(s, "tload"),
            Inst::TStore => write!(s, "tstore"),
            Inst::MCopy => write!(s, "mcopy"),
            Inst::BlobHash => write!(s, "blobhash"),
            Inst::BlobBaseFee => write!(s, "blobbasefee"),
            Inst::Trap { code } => write!(s, "trap {:?}", code),
            Inst::Udf { code } => write!(s, "udf {:?}", code),
        };
        s
    }

    fn canonical_type_for_rc(rc: RegClass) -> Type {
        match rc {
            RegClass::Int => I64,
            RegClass::Float => I64,
            RegClass::Vector => I64,
        }
    }
}

impl MachInst for Inst {
    type ABIMachineSpec = EvmMachineDeps;
    type LabelUse = LabelUse;

    const TRAP_OPCODE: &'static [u8] = &[0xfe];

    fn get_operands(&mut self, _collector: &mut impl OperandVisitor) {}

    fn is_move(&self) -> Option<(Writable<Reg>, Reg)> {
        None
    }

    fn is_included_in_clobbers(&self) -> bool {
        true
    }

    fn is_trap(&self) -> bool {
        matches!(self, Inst::Trap { .. } | Inst::Invalid | Inst::Udf { .. })
    }

    fn is_args(&self) -> bool {
        false
    }

    fn call_type(&self) -> CallType {
        match self {
            Inst::Call | Inst::CallCode | Inst::DelegateCall | Inst::StaticCall => {
                CallType::Regular
            }
            _ => CallType::None,
        }
    }

    fn is_mem_access(&self) -> bool {
        matches!(
            self,
            Inst::MLoad
                | Inst::MStore
                | Inst::MStore8
                | Inst::SLoad
                | Inst::SStore
                | Inst::TLoad
                | Inst::TStore
                | Inst::CallDataLoad
                | Inst::CallDataCopy
                | Inst::CodeCopy
                | Inst::ExtCodeCopy
                | Inst::ReturnDataCopy
                | Inst::MCopy
        )
    }

    fn is_term(&self) -> MachTerminator {
        match self {
            Inst::Jump { .. } | Inst::JumpI { .. } => MachTerminator::Branch,
            Inst::Return | Inst::Revert | Inst::Stop | Inst::Invalid | Inst::SelfDestruct => {
                MachTerminator::Ret
            }
            Inst::Trap { .. } | Inst::Udf { .. } => MachTerminator::Ret,
            _ => MachTerminator::None,
        }
    }

    fn is_safepoint(&self) -> bool {
        false
    }

    fn gen_move(_to_reg: Writable<Reg>, _from_reg: Reg, _ty: Type) -> Self {
        Inst::Nop
    }

    fn gen_nop(_preferred_size: usize) -> Self {
        Inst::Nop
    }

    fn gen_nop_units() -> Vec<Vec<u8>> {
        vec![]
    }

    fn gen_dummy_use(_reg: Reg) -> Self {
        Inst::Nop
    }

    fn rc_for_type(_ty: Type) -> crate::CodegenResult<(&'static [RegClass], &'static [Type])> {
        Ok((&[RegClass::Int], &[I64]))
    }

    fn canonical_type_for_rc(rc: RegClass) -> Type {
        Self::canonical_type_for_rc(rc)
    }

    fn gen_jump(target: MachLabel) -> Self {
        Inst::Jump { target }
    }

    fn worst_case_size() -> CodeOffset {
        33
    }

    fn ref_type_regclass(_settings: &settings::Flags) -> RegClass {
        RegClass::Int
    }

    fn function_alignment() -> FunctionAlignment {
        FunctionAlignment {
            minimum: 1,
            preferred: 1,
        }
    }
}
