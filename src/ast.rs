#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
    Empty,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WasmElemType {
    FuncRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WasmGlobalType {
    Var(WasmType),
    Const(WasmType),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WasmBlockType {
    Empty,
    Valtype(WasmType),
    TypeIndex(i32),
}

// TODO[minor] maybe group these instructions per category in seperate enums
pub enum WasmInstruction {
    // control instructions
    Unreachable,
    Nop,
    Block {
        block_type: WasmBlockType,
        instructions: Vec<WasmInstruction>,
    },
    Loop {
        block_type: WasmBlockType,
        instructions: Vec<WasmInstruction>,
    },
    If {
        block_type: WasmBlockType,
        consequent: Vec<WasmInstruction>,
        alternative: Vec<WasmInstruction>,
    },
    Jump {
        label: u32,
    },
    JumpIf {
        label: u32,
    },
    JumpTable {
        locations: Vec<u32>,
        label: u32,
    },
    Return,
    Call {
        function_index: u32,
    },
    CallIndirect {
        type_index: u32,
    },
    // Parametric instructions
    Drop,
    Select,

    // Variable instructions
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),

    // Memory instructions
    I32Load(WasmMemoryArg),
    I64Load(WasmMemoryArg),
    F32Load(WasmMemoryArg),
    F64Load(WasmMemoryArg),
    I32Load8S(WasmMemoryArg),
    I32Load8U(WasmMemoryArg),
    I32Load16S(WasmMemoryArg),
    I32Load16U(WasmMemoryArg),
    I64Load8S(WasmMemoryArg),
    I64Load8U(WasmMemoryArg),
    I64Load16S(WasmMemoryArg),
    I64Load16U(WasmMemoryArg),
    I64Load32S(WasmMemoryArg),
    I64Load32U(WasmMemoryArg),
    I32Store(WasmMemoryArg),
    I64Store(WasmMemoryArg),
    F32Store(WasmMemoryArg),
    F64Store(WasmMemoryArg),
    I32Store8(WasmMemoryArg),
    I32Store16(WasmMemoryArg),
    I64Store8(WasmMemoryArg),
    I64Store16(WasmMemoryArg),
    I64Store32(WasmMemoryArg),
    MemorySize,
    Memorygrow,

    // numeric instructions
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,

    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,

    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,

    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,

    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,

    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32Copysign,

    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64Copysign,

    I32WrapI64,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DemoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,

    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,

    I32TruncSatF32S,
    I32TruncSatF32U,
    I32TruncSatF64S,
    I32TruncSatF64U,
    I64TruncSatF32S,
    I64TruncSatF32U,
    I64TruncSatF64S,
    I64TruncSatF64U,
}

pub struct WasmMemoryArg {
    align: u32,
    offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmFunctionType {
    pub parameter_types: Vec<WasmType>,
    pub result_types: Vec<WasmType>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmLimitType {
    pub min: u32,
    pub max: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmTableType {
    pub elemtype: WasmElemType,
    pub limits: WasmLimitType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WasmSectionContent<'t> {
    CustomSection {
        name: String,
        bytes: &'t [u8]
    },
    TypeSection {
        types: Vec<WasmFunctionType>
    },
    ImportSection,
    FunctionSection,
    TableSection,
    MemorySection,
    GlobalSection,
    ExportSection,
    StartSection,
    ElementSection,
    CodeSection,
    DataSection,
    UnknownSection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmSection<'t> {
    size: u32,
    content: WasmSectionContent<'t>,
}
