use crate::sir;

pub fn remove_scopes(module: &mut sir::Module) {
    super::transform_module(module, &|expression| match expression {
        sir::Expression::Scope { name, value, body } => {
            remove_scope(name, value, body);
            *expression = *body.clone();
        }
        _ => {}
    })
}

fn remove_scope(name: &str, value: &sir::Expression, body: &mut sir::Expression) {
    super::transform_expression(body, &|expression| match expression {
        sir::Expression::Reference { name: ref_name } if ref_name == name => {
            *expression = value.clone()
        }
        _ => {}
    })
}
