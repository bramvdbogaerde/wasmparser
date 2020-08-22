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
    FuncRef
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WasmGlobalType {
    Var(WasmType),
    Const(WasmType),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmFunctionType {
    pub parameter_types: Vec<WasmType>,
    pub result_types: Vec<WasmType>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmLimitType {
    pub min: u32,
    pub max: Option<u32>,
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WasmTableType {
    pub elemtype: WasmElemType,
    pub limits: WasmLimitType
}


