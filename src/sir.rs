use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Expression {
    BinaryOperation {
        operation: BinaryOperation,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    GlobalReference {
        name: String,
        data_type: DataType,
    },
    I64Literal(i64),
    Lambda {
        arguments: Vec<(String, DataType)>,
        body: Box<Expression>,
    },
    MemberAccess {
        left: Box<Expression>,
        member: String,
    },
    FunctionParam {
        index: u32,
        data_type: DataType,
    },
    Reference {
        name: String,
    },
    Scope {
        name: String,
        value: Box<Expression>,
        body: Box<Expression>,
    },
    Tuple {
        values: Vec<Expression>,
    },
}

#[derive(Clone, Debug)]
pub enum DataType {
    Primitive(PrimitiveDataType),
    Tuple(Vec<DataType>),
}

#[derive(Clone, Debug)]
pub enum PrimitiveDataType {
    Function {
        argument_types: Vec<DataType>,
        return_type: Box<DataType>,
    },
    I64,
}

#[derive(Clone, Debug)]
pub enum BinaryOperation {
    Add,
    Divide,
    Multiply,
    Subtract,
}

#[derive(Debug)]
pub struct Global {
    pub arguments: Vec<(String, DataType)>,
    pub return_type: DataType,
    pub body: Expression,
}

#[derive(Debug)]
pub struct Module {
    pub globals: HashMap<String, Global>,
}
