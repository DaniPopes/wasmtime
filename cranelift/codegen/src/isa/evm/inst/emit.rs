//! EVM bytecode emission.

use super::*;
use crate::isa::evm::settings::Flags as EvmFlags;
use crate::machinst::{MachBuffer, MachInstEmit, MachInstEmitState};
use crate::settings::Flags;
use cranelift_control::ControlPlane;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Opcode {
    Stop = 0x00,
    Add = 0x01,
    Mul = 0x02,
    Sub = 0x03,
    Div = 0x04,
    SDiv = 0x05,
    Mod = 0x06,
    SMod = 0x07,
    AddMod = 0x08,
    MulMod = 0x09,
    Exp = 0x0a,
    SignExtend = 0x0b,

    Lt = 0x10,
    Gt = 0x11,
    SLt = 0x12,
    SGt = 0x13,
    Eq = 0x14,
    IsZero = 0x15,
    And = 0x16,
    Or = 0x17,
    Xor = 0x18,
    Not = 0x19,
    Byte = 0x1a,
    Shl = 0x1b,
    Shr = 0x1c,
    Sar = 0x1d,

    Keccak256 = 0x20,

    Address = 0x30,
    Balance = 0x31,
    Origin = 0x32,
    Caller = 0x33,
    CallValue = 0x34,
    CallDataLoad = 0x35,
    CallDataSize = 0x36,
    CallDataCopy = 0x37,
    CodeSize = 0x38,
    CodeCopy = 0x39,
    GasPrice = 0x3a,
    ExtCodeSize = 0x3b,
    ExtCodeCopy = 0x3c,
    ReturnDataSize = 0x3d,
    ReturnDataCopy = 0x3e,
    ExtCodeHash = 0x3f,

    BlockHash = 0x40,
    Coinbase = 0x41,
    Timestamp = 0x42,
    Number = 0x43,
    PrevRandao = 0x44,
    GasLimit = 0x45,
    ChainId = 0x46,
    SelfBalance = 0x47,
    BaseFee = 0x48,
    BlobHash = 0x49,
    BlobBaseFee = 0x4a,

    Pop = 0x50,
    MLoad = 0x51,
    MStore = 0x52,
    MStore8 = 0x53,
    SLoad = 0x54,
    SStore = 0x55,
    Jump = 0x56,
    JumpI = 0x57,
    Pc = 0x58,
    MSize = 0x59,
    Gas = 0x5a,
    JumpDest = 0x5b,
    TLoad = 0x5c,
    TStore = 0x5d,
    MCopy = 0x5e,

    Push0 = 0x5f,
    Push1 = 0x60,
    Push2 = 0x61,
    Push3 = 0x62,
    Push4 = 0x63,
    Push5 = 0x64,
    Push6 = 0x65,
    Push7 = 0x66,
    Push8 = 0x67,
    Push9 = 0x68,
    Push10 = 0x69,
    Push11 = 0x6a,
    Push12 = 0x6b,
    Push13 = 0x6c,
    Push14 = 0x6d,
    Push15 = 0x6e,
    Push16 = 0x6f,
    Push17 = 0x70,
    Push18 = 0x71,
    Push19 = 0x72,
    Push20 = 0x73,
    Push21 = 0x74,
    Push22 = 0x75,
    Push23 = 0x76,
    Push24 = 0x77,
    Push25 = 0x78,
    Push26 = 0x79,
    Push27 = 0x7a,
    Push28 = 0x7b,
    Push29 = 0x7c,
    Push30 = 0x7d,
    Push31 = 0x7e,
    Push32 = 0x7f,

    Dup1 = 0x80,
    Dup2 = 0x81,
    Dup3 = 0x82,
    Dup4 = 0x83,
    Dup5 = 0x84,
    Dup6 = 0x85,
    Dup7 = 0x86,
    Dup8 = 0x87,
    Dup9 = 0x88,
    Dup10 = 0x89,
    Dup11 = 0x8a,
    Dup12 = 0x8b,
    Dup13 = 0x8c,
    Dup14 = 0x8d,
    Dup15 = 0x8e,
    Dup16 = 0x8f,

    Swap1 = 0x90,
    Swap2 = 0x91,
    Swap3 = 0x92,
    Swap4 = 0x93,
    Swap5 = 0x94,
    Swap6 = 0x95,
    Swap7 = 0x96,
    Swap8 = 0x97,
    Swap9 = 0x98,
    Swap10 = 0x99,
    Swap11 = 0x9a,
    Swap12 = 0x9b,
    Swap13 = 0x9c,
    Swap14 = 0x9d,
    Swap15 = 0x9e,
    Swap16 = 0x9f,

    Log0 = 0xa0,
    Log1 = 0xa1,
    Log2 = 0xa2,
    Log3 = 0xa3,
    Log4 = 0xa4,

    Create = 0xf0,
    Call = 0xf1,
    CallCode = 0xf2,
    Return = 0xf3,
    DelegateCall = 0xf4,
    Create2 = 0xf5,
    StaticCall = 0xfa,
    Revert = 0xfd,
    Invalid = 0xfe,
    SelfDestruct = 0xff,
}

pub struct EmitInfo {
    flags: Flags,
    isa_flags: EvmFlags,
}

impl EmitInfo {
    pub fn new(flags: Flags, isa_flags: EvmFlags) -> Self {
        Self { flags, isa_flags }
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    pub fn isa_flags(&self) -> &EvmFlags {
        &self.isa_flags
    }
}

#[derive(Clone, Debug, Default)]
pub struct EmitState {
    ctrl_plane: ControlPlane,
    frame_layout: FrameLayout,
}

impl MachInstEmitState<Inst> for EmitState {
    fn new(abi: &Callee<EvmMachineDeps>, ctrl_plane: ControlPlane) -> Self {
        Self {
            ctrl_plane,
            frame_layout: abi.frame_layout().clone(),
        }
    }

    fn pre_safepoint(&mut self, _user_stack_map: Option<crate::ir::UserStackMap>) {}

    fn ctrl_plane_mut(&mut self) -> &mut ControlPlane {
        &mut self.ctrl_plane
    }

    fn take_ctrl_plane(self) -> ControlPlane {
        self.ctrl_plane
    }

    fn frame_layout(&self) -> &FrameLayout {
        &self.frame_layout
    }
}

impl MachInstEmit for Inst {
    type State = EmitState;
    type Info = EmitInfo;

    fn emit(&self, sink: &mut MachBuffer<Inst>, emit_info: &Self::Info, _state: &mut Self::State) {
        match self {
            Inst::Nop => {}

            Inst::Stop => {
                sink.put1(0x00);
            }

            Inst::Push { imm } => {
                let leading_zeros = imm.iter().take_while(|&&b| b == 0).count();
                let significant_bytes = &imm[leading_zeros..];

                if significant_bytes.is_empty()
                    || (significant_bytes.len() == 1 && significant_bytes[0] == 0)
                {
                    if emit_info.isa_flags().has_push0() {
                        sink.put1(0x5f);
                    } else {
                        sink.put1(0x60);
                        sink.put1(0x00);
                    }
                } else {
                    sink.put1(0x5f + significant_bytes.len() as u8);
                    for &byte in significant_bytes {
                        sink.put1(byte);
                    }
                }
            }

            Inst::Pop => {
                sink.put1(0x50);
            }

            Inst::Dup { depth } => {
                debug_assert!(*depth >= 1 && *depth <= 16);
                sink.put1(0x7f + *depth);
            }

            Inst::Swap { depth } => {
                debug_assert!(*depth >= 1 && *depth <= 16);
                sink.put1(0x8f + *depth);
            }

            Inst::Add => sink.put1(0x01),
            Inst::Mul => sink.put1(0x02),
            Inst::Sub => sink.put1(0x03),
            Inst::Div => sink.put1(0x04),
            Inst::SDiv => sink.put1(0x05),
            Inst::Mod => sink.put1(0x06),
            Inst::SMod => sink.put1(0x07),
            Inst::AddMod => sink.put1(0x08),
            Inst::MulMod => sink.put1(0x09),
            Inst::Exp => sink.put1(0x0a),
            Inst::SignExtend => sink.put1(0x0b),

            Inst::Lt => sink.put1(0x10),
            Inst::Gt => sink.put1(0x11),
            Inst::SLt => sink.put1(0x12),
            Inst::SGt => sink.put1(0x13),
            Inst::Eq => sink.put1(0x14),
            Inst::IsZero => sink.put1(0x15),

            Inst::And => sink.put1(0x16),
            Inst::Or => sink.put1(0x17),
            Inst::Xor => sink.put1(0x18),
            Inst::Not => sink.put1(0x19),
            Inst::Byte => sink.put1(0x1a),
            Inst::Shl => sink.put1(0x1b),
            Inst::Shr => sink.put1(0x1c),
            Inst::Sar => sink.put1(0x1d),

            Inst::Keccak256 => sink.put1(0x20),

            Inst::MLoad => sink.put1(0x51),
            Inst::MStore => sink.put1(0x52),
            Inst::MStore8 => sink.put1(0x53),
            Inst::SLoad => sink.put1(0x54),
            Inst::SStore => sink.put1(0x55),

            Inst::Jump { target } => {
                sink.put1(0x56);
                sink.use_label_at_offset(sink.cur_offset(), *target, LabelUse::Jump);
            }

            Inst::JumpI { target } => {
                sink.put1(0x57);
                sink.use_label_at_offset(sink.cur_offset(), *target, LabelUse::Jump);
            }

            Inst::JumpDest => {
                sink.put1(0x5b);
            }

            Inst::Pc => sink.put1(0x58),
            Inst::MSize => sink.put1(0x59),
            Inst::Gas => sink.put1(0x5a),

            Inst::Return => sink.put1(0xf3),
            Inst::Revert => sink.put1(0xfd),
            Inst::Invalid => sink.put1(0xfe),
            Inst::SelfDestruct => sink.put1(0xff),

            Inst::Address => sink.put1(0x30),
            Inst::Balance => sink.put1(0x31),
            Inst::Origin => sink.put1(0x32),
            Inst::Caller => sink.put1(0x33),
            Inst::CallValue => sink.put1(0x34),
            Inst::CallDataLoad => sink.put1(0x35),
            Inst::CallDataSize => sink.put1(0x36),
            Inst::CallDataCopy => sink.put1(0x37),
            Inst::CodeSize => sink.put1(0x38),
            Inst::CodeCopy => sink.put1(0x39),
            Inst::GasPrice => sink.put1(0x3a),
            Inst::ExtCodeSize => sink.put1(0x3b),
            Inst::ExtCodeCopy => sink.put1(0x3c),
            Inst::ReturnDataSize => sink.put1(0x3d),
            Inst::ReturnDataCopy => sink.put1(0x3e),
            Inst::ExtCodeHash => sink.put1(0x3f),

            Inst::BlockHash => sink.put1(0x40),
            Inst::Coinbase => sink.put1(0x41),
            Inst::Timestamp => sink.put1(0x42),
            Inst::Number => sink.put1(0x43),
            Inst::PrevRandao => sink.put1(0x44),
            Inst::GasLimit => sink.put1(0x45),
            Inst::ChainId => sink.put1(0x46),
            Inst::SelfBalance => sink.put1(0x47),
            Inst::BaseFee => sink.put1(0x48),

            Inst::Log { topic_count } => {
                debug_assert!(*topic_count <= 4);
                sink.put1(0xa0 + *topic_count);
            }

            Inst::Call => sink.put1(0xf1),
            Inst::CallCode => sink.put1(0xf2),
            Inst::DelegateCall => sink.put1(0xf4),
            Inst::StaticCall => sink.put1(0xfa),
            Inst::Create => sink.put1(0xf0),
            Inst::Create2 => sink.put1(0xf5),

            Inst::TLoad => sink.put1(0x5c),
            Inst::TStore => sink.put1(0x5d),
            Inst::MCopy => sink.put1(0x5e),
            Inst::BlobHash => sink.put1(0x49),
            Inst::BlobBaseFee => sink.put1(0x4a),

            Inst::Trap { code } => {
                sink.add_trap(*code);
                sink.put1(0xfe);
            }

            Inst::Udf { code } => {
                sink.add_trap(*code);
                sink.put1(0xfe);
            }
        }
    }

    fn pretty_print_inst(&self, _state: &mut Self::State) -> alloc::string::String {
        self.print_with_state(&mut EmitState {
            ctrl_plane: ControlPlane::default(),
            frame_layout: FrameLayout::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings;
    use crate::machinst::VCodeConstants;

    fn emit_inst(inst: &Inst, has_push0: bool) -> Vec<u8> {
        let shared_flags = settings::Flags::new(settings::builder());
        let isa_flags = EvmFlags::new(&shared_flags, &crate::isa::evm::settings::builder());
        let emit_info = EmitInfo::new(shared_flags, isa_flags);
        let mut state = EmitState::default();
        let mut buffer = MachBuffer::new();
        inst.emit(&mut buffer, &emit_info, &mut state);
        let _ = has_push0;
        let constants = VCodeConstants::default();
        buffer.finish(&constants, &mut Default::default()).data().to_vec()
    }

    #[test]
    fn test_simple_opcodes() {
        assert_eq!(emit_inst(&Inst::Stop, true), &[Opcode::Stop as u8]);
        assert_eq!(emit_inst(&Inst::Add, true), &[Opcode::Add as u8]);
        assert_eq!(emit_inst(&Inst::Sub, true), &[Opcode::Sub as u8]);
        assert_eq!(emit_inst(&Inst::Mul, true), &[Opcode::Mul as u8]);
        assert_eq!(emit_inst(&Inst::Div, true), &[Opcode::Div as u8]);
        assert_eq!(emit_inst(&Inst::Pop, true), &[Opcode::Pop as u8]);
        assert_eq!(emit_inst(&Inst::Return, true), &[Opcode::Return as u8]);
        assert_eq!(emit_inst(&Inst::Revert, true), &[Opcode::Revert as u8]);
        assert_eq!(emit_inst(&Inst::Invalid, true), &[Opcode::Invalid as u8]);
    }

    #[test]
    fn test_dup_swap() {
        assert_eq!(emit_inst(&Inst::Dup { depth: 1 }, true), &[Opcode::Dup1 as u8]);
        assert_eq!(emit_inst(&Inst::Dup { depth: 16 }, true), &[Opcode::Dup16 as u8]);
        assert_eq!(emit_inst(&Inst::Swap { depth: 1 }, true), &[Opcode::Swap1 as u8]);
        assert_eq!(emit_inst(&Inst::Swap { depth: 16 }, true), &[Opcode::Swap16 as u8]);
    }

    #[test]
    fn test_log() {
        assert_eq!(emit_inst(&Inst::Log { topic_count: 0 }, true), &[Opcode::Log0 as u8]);
        assert_eq!(emit_inst(&Inst::Log { topic_count: 4 }, true), &[Opcode::Log4 as u8]);
    }

    #[test]
    fn test_push_zero() {
        let imm = [0u8; 32];
        let bytes = emit_inst(&Inst::Push { imm }, true);
        assert_eq!(bytes, &[Opcode::Push0 as u8]);
    }

    #[test]
    fn test_push_small() {
        let mut imm = [0u8; 32];
        imm[31] = 0x42;
        let bytes = emit_inst(&Inst::Push { imm }, true);
        assert_eq!(bytes, &[Opcode::Push1 as u8, 0x42]);
    }

    #[test]
    fn test_push_two_bytes() {
        let mut imm = [0u8; 32];
        imm[30] = 0x01;
        imm[31] = 0x00;
        let bytes = emit_inst(&Inst::Push { imm }, true);
        assert_eq!(bytes, &[Opcode::Push2 as u8, 0x01, 0x00]);
    }

    #[test]
    fn test_push_max() {
        let imm = [0xffu8; 32];
        let bytes = emit_inst(&Inst::Push { imm }, true);
        assert_eq!(bytes.len(), 33);
        assert_eq!(bytes[0], Opcode::Push32 as u8);
        assert!(bytes[1..].iter().all(|&b| b == 0xff));
    }

    #[test]
    fn test_comparison_ops() {
        assert_eq!(emit_inst(&Inst::Lt, true), &[Opcode::Lt as u8]);
        assert_eq!(emit_inst(&Inst::Gt, true), &[Opcode::Gt as u8]);
        assert_eq!(emit_inst(&Inst::Eq, true), &[Opcode::Eq as u8]);
        assert_eq!(emit_inst(&Inst::IsZero, true), &[Opcode::IsZero as u8]);
    }

    #[test]
    fn test_bitwise_ops() {
        assert_eq!(emit_inst(&Inst::And, true), &[Opcode::And as u8]);
        assert_eq!(emit_inst(&Inst::Or, true), &[Opcode::Or as u8]);
        assert_eq!(emit_inst(&Inst::Xor, true), &[Opcode::Xor as u8]);
        assert_eq!(emit_inst(&Inst::Not, true), &[Opcode::Not as u8]);
        assert_eq!(emit_inst(&Inst::Shl, true), &[Opcode::Shl as u8]);
        assert_eq!(emit_inst(&Inst::Shr, true), &[Opcode::Shr as u8]);
    }

    #[test]
    fn test_memory_ops() {
        assert_eq!(emit_inst(&Inst::MLoad, true), &[Opcode::MLoad as u8]);
        assert_eq!(emit_inst(&Inst::MStore, true), &[Opcode::MStore as u8]);
        assert_eq!(emit_inst(&Inst::SLoad, true), &[Opcode::SLoad as u8]);
        assert_eq!(emit_inst(&Inst::SStore, true), &[Opcode::SStore as u8]);
    }

    #[test]
    fn test_env_ops() {
        assert_eq!(emit_inst(&Inst::Address, true), &[Opcode::Address as u8]);
        assert_eq!(emit_inst(&Inst::Caller, true), &[Opcode::Caller as u8]);
        assert_eq!(emit_inst(&Inst::CallValue, true), &[Opcode::CallValue as u8]);
        assert_eq!(emit_inst(&Inst::Gas, true), &[Opcode::Gas as u8]);
    }
}
