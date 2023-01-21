use crate::sir;

pub fn build_function_params(module: &mut sir::Module) {
    for global in module.globals.values_mut() {
        super::transform_expression(&mut global.body, &|expression| match expression {
            sir::Expression::Reference { name } => {
                let target = global
                    .arguments
                    .iter()
                    .enumerate()
                    .filter(|(_, (argument_name, _))| argument_name == name)
                    .map(|(index, (_, data_type))| sir::Expression::FunctionParam {
                        index: index as u32,
                        data_type: data_type.clone(),
                    })
                    .next();
                if let Some(argument) = target {
                    *expression = argument
                }
            }
            _ => {}
        });
    }
}
