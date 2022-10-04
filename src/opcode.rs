type Address = u16;
type Const8 = u8;
type Const4 = u8;
type RegId = u8;

// All 35 opcodes
pub enum Opcode {
    MachineCode(Address),
    ClearScreen,
    ReturnFromSub,
    Goto(Address),
    CallSub(Address),
    SkipEQ(RegId, Const8),
    SkipNEQ(RegId, Const8),
    SkipRegEQ(RegId, RegId),
    SetConst(RegId, Const8),
    AddConst(RegId, Const8),
    SetReg(RegId, RegId),
    Or(RegId, RegId),
    And(RegId, RegId),
    Xor(RegId, RegId),
    AddReg(RegId, RegId),
    SubReg(RegId, RegId),
    Div2(RegId, RegId),
    DiffReg(RegId, RegId),
    Mul2(RegId, RegId),
    SkipRegNEQ(RegId, RegId),
    SetAR(Address),
    Jump(Address),
    Rand(RegId, Const8),
    Draw(RegId, RegId, Const4),
    KeyEQ(RegId),
    KeyNEQ(RegId),
    GetDelayTimer(RegId),
    GetKey(RegId),
    SetDelayTimer(RegId),
    SetSoundTimer(RegId),
    AddToI(RegId),
    SetISprite(RegId),
    BCD(RegId), // no clue rn - trying to understand a lecture while writing this
    RegDump(RegId),
    RegLoad(RegId),
}

mod nibble {
    enum Nibble {
        x0,
        x1,
        x2,
        x3,
        x4,
        x5,
        x6,
        x7,
        x8,
        x9,
        xA,
        xB,
        xC,
        xD,
        xE,
        xF,
    }

    fn split_u16(value: u16) -> (Nibble, Nibble, Nibble, Nibble) {
        (
            ((value >> 24) & 0xF).try_into().unwrap(),
            ((value >> 16) & 0xF).try_into().unwrap(),
            ((value >> 8) & 0xF).try_into().unwrap(),
            ((value >> 0) & 0xF).try_into().unwrap(),
        )
    }
}

pub fn from_value(value: u16) -> Opcode {
    use nibbles::Nibble::*;
    let nibbles = nibble::split_u16(value);
    match nibbles {
        (x0, n1, n2, n3) => 
        _ => panic!();
    }
    Opcode::ClearScreen
}
