use std::path::Path;

use inkwell::context::Context;
use nom::Parser;

use crate::{
    generator::Generator,
    parser::module,
    passes::{
        build_function_params::build_function_params,
        build_global_references::build_global_references, remove_scopes::remove_scopes,
    },
};

mod generator;
mod parser;
mod passes;
mod sir;

fn main() -> anyhow::Result<()> {
    let text = r#"
add(pair: (I64, I64)): I64 = pair.elem_0 + pair.elem_1
    "#;

    let parsed = module.parse(text)?;
    println!("{:#?}", parsed);
    let mut parsed = parsed.1;

    remove_scopes(&mut parsed);
    build_function_params(&mut parsed);
    build_global_references(&mut parsed);
    println!("{:#?}", parsed);

    let context = Context::create();
    let mut generator = Generator::new(&context);

    for (name, global) in parsed.globals.iter() {
        if global.arguments.is_empty() {
            generator.declare_global_constant(name.clone(), &global.return_type);
        } else {
            generator.declare_global_function(name.clone(), &global.arguments, &global.return_type);
        }
    }

    for (name, global) in parsed.globals.iter() {
        if global.arguments.is_empty() {
            if let sir::DataType::Primitive(_) = global.return_type {
                generator.write_global_primitive_constant(name, &global.body);
            } else {
                generator.write_global_nonprimitive_constant(name, &global.body);
            }
        } else {
            generator.write_global_function(name, &global.body);
        }
    }

    let module = generator.build();
    println!("{}", module.to_string());
    module.write_bitcode_to_path(&Path::new("scrap.ll"));

    Ok(())
}
