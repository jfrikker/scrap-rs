use std::{borrow::Cow, collections::HashMap, fmt::{Write, self}};

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

impl Expression {
    pub fn data_type(&self) -> Cow<DataType> {
        match self {
            Expression::BinaryOperation { left, .. } => left.data_type(),
            Expression::Call { function, .. } => {
                let return_type = function.data_type();
                let DataType::Primitive(PrimitiveDataType::Function { return_type, .. }) = return_type.as_ref() else {panic!("Non-function")};
                Cow::Owned(return_type.as_ref().clone())
            }
            Expression::GlobalReference { data_type, .. } => Cow::Borrowed(data_type),
            Expression::I64Literal(_) => Cow::Owned(DataType::Primitive(PrimitiveDataType::I64)),
            Expression::MemberAccess { left, member } => Cow::Owned(left.data_type().field_type(&member).unwrap().clone()),
            Expression::FunctionParam { data_type, .. } => Cow::Borrowed(data_type),
            Expression::Reference { .. } => todo!(),
            Expression::Scope { body, .. } => body.data_type(),
            Expression::Tuple { values } => Cow::Owned(DataType::Tuple(
                values.iter().map(|value| value.data_type().into_owned()).collect(),
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DataType {
    Primitive(PrimitiveDataType),
    Tuple(Vec<DataType>),
}

impl DataType {
    pub fn is_primitive(&self) -> bool {
        matches!(self, DataType::Primitive(_))
    }

    pub fn mangle(&self, out: &mut impl Write) -> fmt::Result {
        match self {
            DataType::Primitive(t) => t.mangle(out),
            DataType::Tuple(elements) => {
                write!(out, "{{")?;
                let mut first = true;
                for element in elements {
                    if first {
                        first = false;
                    } else {
                        write!(out, ",")?;
                    }
                    element.mangle(out)?;
                }
                write!(out, "}}")?;
                Ok(())
            }
        }
    }

    pub fn fields(&self) -> Vec<(Cow<String>, &DataType)> {
        match self {
            DataType::Primitive(_) => Vec::new(),
            DataType::Tuple(elems) => elems.iter()
                .enumerate()
                .map(|(i, data_type)| (Cow::Owned(format!("elem_{}", i)), data_type))
                .collect(),
        }
    }

    pub fn field_type(&self, name: &str) -> Option<&DataType> {
        self.fields().into_iter()
            .find(|(element_name, _)| element_name.as_ref() == name)
            .map(|(_, data_type)| data_type)
    }

    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.fields().into_iter()
            .enumerate()
            .find(|(_, (element_name, _))| element_name.as_ref() == name)
            .map(|(i, _)| i)
    }
}

#[derive(Clone, Debug)]
pub enum PrimitiveDataType {
    Function {
        argument_types: Vec<DataType>,
        return_type: Box<DataType>,
    },
    I64,
}

impl PrimitiveDataType {
    fn mangle(&self, out: &mut impl Write) -> fmt::Result {
        match self {
            PrimitiveDataType::Function { .. } => todo!(),
            PrimitiveDataType::I64 => write!(out, "I64"),
        }
    }
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
