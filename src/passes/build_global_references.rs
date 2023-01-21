use std::collections::HashMap;

use crate::sir;

pub fn build_global_references(module: &mut sir::Module) {
    let global_types: HashMap<_, _> = module.globals.iter()
        .map(|(name, global)| (name.clone(), global_type(global)))
        .collect();

    for global in module.globals.values_mut() {
        super::transform_expression(&mut global.body, &|expression| match expression {
            sir::Expression::Reference { name } => {
                if let Some(data_type) = global_types.get(&*name) {
                    *expression = sir::Expression::GlobalReference { name: name.clone(), data_type: data_type.clone() };
                }
            }
            _ => {}
        });
    }
}

fn global_type(global: &sir::Global) -> sir::DataType {
    if global.arguments.is_empty() {
        global.return_type.clone()
    } else {
        let argument_types = global.arguments.iter()
            .map(|(_, argument_type)| argument_type.clone())
            .collect();
        sir::DataType::Primitive(sir::PrimitiveDataType::Function { argument_types, return_type: Box::new(global.return_type.clone()) })
    }
}
