use std::path::Path;

use inkwell::context::Context;
use nom::Parser;

use crate::{parser::module, generator::Generator, passes::remove_scopes::remove_scopes};

mod generator;
mod parser;
mod passes;
mod sir;

fn main() -> anyhow::Result<()> {
    let text = r#"
test: ((I64, I64), I64, (I64, I64, I64)) = ((123i64 + 4i64, 456i64), 1i64, (12i64, 54i64, 23i64))
    "#;

    let parsed = module.parse(text)?;
    println!("{:#?}", parsed);
    let mut parsed = parsed.1;

    remove_scopes(&mut parsed);
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
            generator.write_global_function(name, &global.arguments, &global.body);
        }
    }

    let module = generator.build();
    println!("{}", module.to_string());
    module.write_bitcode_to_path(&Path::new("scrap.ll"));

    Ok(())
}
