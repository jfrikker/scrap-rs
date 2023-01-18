use crate::sir;

pub mod remove_scopes;

pub fn transform_module(module: &mut sir::Module, f: &impl Fn(&mut sir::Expression)) {
    for global in module.globals.values_mut() {
        transform_expression(&mut global.body, f);
    }
}

pub fn transform_expression(expression: &mut sir::Expression, f: &impl Fn(&mut sir::Expression)) {
    match expression {
        sir::Expression::BinaryOperation { left, right, .. } => {
            transform_expression(left, f);
            transform_expression(right, f);
        }
        sir::Expression::Call { function, arguments } => {
            transform_expression(function, f);
            for argument in arguments {
                transform_expression(argument, f);
            }
        }
        sir::Expression::I64Literal(_) => {}
        sir::Expression::Lambda { body, .. } => {
            transform_expression(body, f);
        }
        sir::Expression::MemberAccess { left, .. } => {
            transform_expression(left, f);
        }
        sir::Expression::Reference { .. } => {}
        sir::Expression::Scope { value, body, .. } => {
            transform_expression(value, f);
            transform_expression(body, f);
        }
    }

    f(expression);
}