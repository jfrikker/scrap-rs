use std::rc::Rc;

use crate::sir;

pub mod remove_scopes;

pub fn transform_expression(expression: Rc<sir::Expression>, f: impl Fn(Rc<sir::Expression>) -> Rc<sir::Expression>) -> Rc<sir::Expression> {
    match &*expression {
        sir::Expression::BinaryOperation { operation, left, right } => {
            let left = transform_expression(left.clone(), f);
            let right = transform_expression(right.clone(), f);
            Rc::new()
        },
        sir::Expression::Call { function, arguments } => todo!(),
        sir::Expression::I64Literal(_) => todo!(),
        sir::Expression::Lambda { arguments, body } => todo!(),
        sir::Expression::MemberAccess { left, member } => todo!(),
        sir::Expression::Reference { name } => todo!(),
        sir::Expression::Scope { name, value, body } => todo!(),
    }
}