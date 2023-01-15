use std::{rc::Rc, collections::HashMap};

#[derive(Debug)]
pub enum Expression {
    BinaryOperation {
        operation: BinaryOperation,
        left: Rc<Expression>,
        right: Rc<Expression>,
    },
    Call {
        function: Rc<Expression>,
        arguments: Vec<Rc<Expression>>,
    },
    GlobalReference {
        name: Rc<String>,
        dataType: Rc<DataType>,
    },
    I64Literal(i64),
    Lambda {
        arguments: Vec<(Rc<String>, Rc<DataType>)>,
        body: Rc<Expression>,
    },
    MemberAccess {
        left: Rc<Expression>,
        member: Rc<String>,
    },
    Reference {
        name: Rc<String>,
    },
    Scope {
        name: Rc<String>,
        value: Rc<Expression>,
        body: Rc<Expression>,
    }
}

#[derive(Debug)]
pub enum DataType {
    Function {
        argument_types: Vec<Rc<DataType>>,
        return_type: Rc<DataType>,
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