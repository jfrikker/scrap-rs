use std::{rc::Rc, collections::HashMap};

#[derive(Debug)]
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
    I64Literal(i64),
    Lambda {
        arguments: Vec<(String, DataType)>,
        body: Box<Expression>,
    },
    MemberAccess {
        left: Box<Expression>,
        member: Box<String>,
    },
    Reference {
        name: String,
    },
    Scope {
        name: String,
        value: Box<Expression>,
        body: Box<Expression>,
    }
}

#[derive(Debug)]
pub enum DataType {
    Primitive(PrimitiveDataType),
}

#[derive(Debug)]
pub enum PrimitiveDataType {
    Function {
        argument_types: Vec<DataType>,
        return_type: Box<DataType>,
    },
    I64,
}

#[derive(Debug)]
pub enum BinaryOperation {
    Add,
    Divide,
    Multiply,
    Subtract,
}

#[derive(Debug)]
pub struct Global {
    pub arguments: Vec<(Rc<String>, Rc<DataType>)>,
    pub return_type: Rc<DataType>,
    pub body: Rc<Expression>,
}

#[derive(Debug)]
pub struct Module {
    pub globals: HashMap<Rc<String>, Rc<Global>>,
}
