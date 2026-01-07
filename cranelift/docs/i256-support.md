# I256 Support in Cranelift IR

## Overview

This document describes the additions made to Cranelift to support 256-bit integers (`I256`) at the IR level. This includes a proper `I256` wrapper type with arithmetic operations, enabling future constant folding support.

## Changes Made

### 1. Type System (Meta Crate)

**File: `codegen/meta/src/shared/types.rs`**
- Added `I256 = 256` variant to the `Int` enum.
- Updated `IntIterator` to include `I256` in iteration.

**File: `codegen/meta/src/cdsl/types.rs`**
- Added type number assignment for `I256` (number 13).
- Updated `int_from_bits` to handle 256-bit integers.

**File: `codegen/meta/src/cdsl/typevar.rs`**
- Updated `MAX_BITS` from 128 to 256 to allow I256 in type sets.

**File: `codegen/meta/src/gen_inst.rs`**
- Updated `ints` bitset generation from 8-bit to 16-bit to accommodate I256.

### 2. Runtime Type Implementation

**File: `codegen/src/ir/types.rs`**
- Added `I256` handling in all `Type` methods:
  - `log2_lane_bits()`: Returns 8 for I256.
  - `lane_bits()`: Returns 256 for I256.
  - `int()`: Maps 256 bits to `Some(I256)`.
  - `half_width()`: I256 → I128.
  - `double_width()`: I128 → I256 (I256 has no double_width).
  - `is_int()`: Returns true for I256.

### 3. I256 Wrapper Type

**File: `codegen/src/ir/i256.rs`** (NEW)
- Created `I256` wrapper type around `[u64; 4]` in little-endian order.
- Implements standard library-like APIs:

**Arithmetic Operations:**
- `wrapping_add`, `wrapping_sub`, `wrapping_mul`, `wrapping_neg`

**Bitwise Operations:**
- `bitand`, `bitor`, `bitxor`, `not`
- `wrapping_shl`, `wrapping_ushr`, `wrapping_sshr`

**Comparison Operations:**
- `cmp_signed`, `cmp_unsigned`
- Implements `Ord`, `PartialOrd`, `Eq`, `PartialEq`

**Utility Operations:**
- `leading_zeros`, `trailing_zeros`, `count_ones`
- `swap_bytes`, `is_zero`, `is_negative`

**Conversions:**
- `From<i64>`, `From<u64>`, `From<i128>`, `From<u128>`
- `From<[u8; 32]>`, `From<[u64; 4]>`
- `from_le_bytes`, `from_be_bytes`, `to_le_bytes`, `to_be_bytes`

**Traits:**
- `Clone`, `Copy`, `Debug`, `Display`, `Hash`, `Default`
- `Add`, `Sub`, `BitAnd`, `BitOr`, `BitXor`, `Not`, `Shl`, `Shr`

### 4. DataValue Support

**File: `codegen/src/data_value.rs`**
- Updated `I256(I256)` variant to use the new wrapper type instead of `[u8; 32]`.
- All trait methods work with the I256 wrapper:
  - `swap_bytes()` uses `I256::swap_bytes()`
  - `write_to_slice_ne()` uses `I256::to_le_bytes()`
  - `read_from_slice_ne()` uses `I256::from_le_bytes()`

### 5. ValueTypeSet Updates

**File: `codegen/src/ir/instructions.rs`**
- Changed `ValueTypeSet.ints` from `BitSet8` to `BitSet16`.
- Updated `Narrower` and `Wider` constraint handling for I256.
- Updated tests to use `BitSet16` for ints.

### 6. Parser Support

**File: `reader/src/lexer.rs`**
- Added `"i256"` to the type lexer, mapping to `types::I256`.

**File: `reader/src/parser.rs`**
- Added I256 parsing using `I256::from_le_bytes()`.
- I256 values are parsed as hexadecimal constants (32 bytes).

### 7. ISLE Prelude

**File: `codegen/src/prelude.isle`**
- Added `(extern const $I256 Type)` to expose I256 in ISLE rules.

### 8. Interpreter Support

**File: `interpreter/src/value.rs`**
- Added `I256` import and basic support:
  - `is_zero()` uses `I256::is_zero()`
  - Bitwise operations work on I256 limbs

### 9. Parser Filetest

**File: `filetests/filetests/parser/i256.clif`** (NEW)
- Tests I256 parameter passing and return values.
- Tests arithmetic operations (iadd, isub, imul).
- Tests bitwise operations (band, bor, bxor, bnot).
- Tests shift operations (ishl, ushr, sshr).
- Tests comparison operations (icmp eq/ne/slt/ult).
- Tests extend/reduce operations (sextend, uextend, ireduce).

## What Works

1. **Type Declaration**: The `i256` type is recognized in CLIF text format.
2. **Type Queries**: All `Type` methods work correctly with I256.
3. **I256 Arithmetic**: The wrapper type supports all arithmetic operations needed for constant folding.
4. **DataValue**: I256 values can be created, compared, serialized, and displayed.
5. **Parsing**: CLIF reader can parse `i256` types and hexadecimal I256 literals.
6. **ISLE Rules**: The `$I256` type is available for pattern matching in ISLE.
7. **IR Instructions**: I256 can be used as a typevar for instructions like iadd, isub, imul, band, bor, etc.

## What's NOT Implemented (TODOs)

### Backend Lowering
- No machine instruction lowering for I256 operations on any target (x64, aarch64, riscv64, s390x, pulley).
- Attempting to compile functions with I256 operations will fail at the lowering stage.

### Constant Folding (cprop.isle)
- **IMPLEMENTED**: I256 constants that fit in 64 bits are folded at compile time.
- The optimizer matches extension chains (`uextend.i256 (uextend.i128 (iconst.i64 k))`) and folds arithmetic operations (iadd, isub, imul) and bitwise operations (band, bor, bxor).
- Overflow is handled by checking if the result fits in u64; if not, the operation is left unfolded.
- Full I256 arithmetic (for constants larger than 64 bits) is not yet implemented.

### Type Bounds
- `Type::bounds()` does not handle I256 because it returns `(u128, u128)`.

### Vector Types
- No I256 vector types (I256X2, etc.) since I256 cannot be a SIMD lane type.

### Decimal Parsing
- Only hex literals work for I256 (no decimal parsing implemented).

## Usage Example

```clif
function %add_i256(i256, i256) -> i256 {
block0(v0: i256, v1: i256):
    v2 = iadd v0, v1
    return v2
}
```

Note: This function can be parsed and verified but cannot be compiled to machine code.

## Testing

```bash
# Unit tests for I256 wrapper
cargo test -p cranelift-codegen --features all-arch -- i256

# All codegen tests
cargo test -p cranelift-codegen --features all-arch

# Parser filetests including I256
cargo run -p cranelift-tools -- test filetests/filetests/parser/i256.clif

# I256 optimizer tests
cargo run -p cranelift-tools -- test filetests/filetests/egraph/i256-opts.clif
```

## Future Work

1. **Full I256 Constant Folding**: Add I256 rules for constants larger than 64 bits using the wrapper's arithmetic methods.

2. **Backend Lowering**: Add I256 lowering rules for each target ISA, decomposing into 64-bit or 128-bit operations.

3. **Interpreter**: Extend interpreter to fully handle I256 arithmetic operations.

4. **Decimal Parsing**: Add support for parsing decimal I256 literals.

5. **Division/Remainder**: Implement `wrapping_div` and `wrapping_rem` for the I256 type.
