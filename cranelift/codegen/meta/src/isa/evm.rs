//! EVM (Ethereum Virtual Machine) Settings.

use crate::cdsl::isa::TargetIsa;
use crate::cdsl::settings::SettingGroupBuilder;

pub(crate) fn define() -> TargetIsa {
    let mut setting = SettingGroupBuilder::new("evm");

    let _has_push0 = setting.add_bool(
        "has_push0",
        "has PUSH0 opcode?",
        "PUSH0: Push 0 onto stack (EIP-3855, Shanghai upgrade)",
        true,
    );

    let _has_mcopy = setting.add_bool(
        "has_mcopy",
        "has MCOPY opcode?",
        "MCOPY: Memory copy (EIP-5656, Cancun upgrade)",
        true,
    );

    let _has_tstore = setting.add_bool(
        "has_tstore",
        "has TSTORE/TLOAD opcodes?",
        "Transient storage opcodes (EIP-1153, Cancun upgrade)",
        true,
    );

    let _has_blobhash = setting.add_bool(
        "has_blobhash",
        "has BLOBHASH opcode?",
        "BLOBHASH: Get versioned hash (EIP-4844, Cancun upgrade)",
        true,
    );

    TargetIsa::new("evm", setting.build())
}
