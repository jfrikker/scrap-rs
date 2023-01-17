use std::{collections::HashMap, rc::Rc};

use inkwell::{context::Context, module::Module, builder::Builder, values::{BasicValue, BasicValueEnum}, types::{BasicTypeEnum, BasicType}};

use crate::sir;

pub struct Generator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    expression_scope: HashMap<Rc<String>, BasicValueEnum<'ctx>>,
}

impl <'ctx> Generator<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            module: context.create_module("scrap"),
            builder: context.create_builder(),
            expression_scope: HashMap::new(),
        }
    }

    pub fn write_global_primitive_constant(&mut self, name: &str, data_type: sir::PrimitiveDataType, value: &sir::Expression) {
        let func_type = self.primitive_type_to_llvm(data_type).fn_type(&[], false);
        let func = self.module.add_function(name, func_type, None);

        let entry_block = self.context.append_basic_block(func, "entry");

        self.builder.position_at_end(entry_block);
        let result = self.write_primitive_expression(value);
        self.builder.build_return(Some(&result));
    }

    pub fn write_global_function(&mut self, name: &str, arguments: &[(Rc<String>, Rc<sir::DataType>)], return_type: &sir::DataType, value: &sir::Expression) {
        let return_type = if let sir::DataType::Primitive(return_type) = return_type {
            *return_type
        } else {
            todo!()
        };

        let param_types: Vec<_> = arguments.iter()
            .map(|(_, data_type)| {
                let data_type = if let sir::DataType::Primitive(data_type) = data_type.as_ref() {
                    data_type
                } else {
                    todo!()
                };
                self.primitive_type_to_llvm(*data_type).into()
            })
            .collect();

        let func_type = self.primitive_type_to_llvm(return_type).fn_type(&param_types, false);
        let func = self.module.add_function(name, func_type, None);

        for (i, (name, _)) in arguments.into_iter().enumerate() {
            self.expression_scope.insert(name.clone(), func.get_nth_param(i as u32).unwrap());
        }

        let entry_block = self.context.append_basic_block(func, "entry");

        self.builder.position_at_end(entry_block);
        let result = self.write_primitive_expression(value);
        self.builder.build_return(Some(&result));

        self.expression_scope.clear();
    }

    // pub fn write_global_primitive_constant(&self, name: &str, data_type: &sir::DataType, value: &sir::Expression) {
    //     let initialized_name = format!("{}$initialized", name);
    //     let value_name = format!("{}$value", name);
    //     let initialized = self.module.add_global(self.context.bool_type(), None, &initialized_name);
    //     initialized.set_initializer(&self.context.bool_type().const_zero());
    //     let value = self.module.add_global(self.context.i64_type(), None, &value_name);

    //     let func_type = self.context.i64_type().fn_type(&[], false);
    //     let func = self.module.add_function(name, func_type, None);

    //     let builder = self.context.create_builder();
    //     let entry_block = self.context.append_basic_block(func, "entry");
    //     let initialize_block = self.context.append_basic_block(func, "initialize");
    //     let return_block = self.context.append_basic_block(func, "");

    //     builder.position_at_end(entry_block);
    //     let initialized_value = builder.build_load(initialized.as_pointer_value(), "initialized").into_int_value();
    //     builder.build_conditional_branch(initialized_value, return_block, initialize_block);

    //     builder.position_at_end(initialize_block);
    //     builder.build_store(value.as_pointer_value(), self.context.i64_type().const_int(123, false));
    //     builder.build_store(initialized.as_pointer_value(), self.context.bool_type().const_all_ones());
    //     builder.build_unconditional_branch(return_block);

    //     builder.position_at_end(return_block);
    //     let value_value = builder.build_load(value.as_pointer_value(), "result");
    //     builder.build_return(Some(&value_value));
    // }

    fn write_primitive_expression(&mut self, expr: &sir::Expression) -> BasicValueEnum<'ctx> {
        match expr {
            sir::Expression::I64Literal(val) => self.context.i64_type().const_int(*val as u64, true).as_basic_value_enum(),
            sir::Expression::BinaryOperation {
                operation: sir::BinaryOperation::Add,
                left,
                right
            } => {
                let left = self.write_primitive_expression(left.as_ref()).into_int_value();
                let right = self.write_primitive_expression(right.as_ref()).into_int_value();
                self.builder.build_int_add(left, right, "add").as_basic_value_enum()
            }
            sir::Expression::Reference { name } => self.expression_scope.get(name).unwrap().clone(),
            sir::Expression::Scope {
                name,
                value,
                body
            } => {
                let new_val = self.write_primitive_expression(value.as_ref());
                let old_val = self.expression_scope.insert(name.clone(), new_val);
                let result = self.write_primitive_expression(body);
                if let Some(old_val) = old_val {
                    self.expression_scope.insert(name.clone(), old_val);
                } else {
                    self.expression_scope.remove(name);
                }
                result.as_basic_value_enum()
            }
            _ => todo!(),
        }
    }

    fn primitive_type_to_llvm(&self, data_type: sir::PrimitiveDataType) -> BasicTypeEnum<'ctx> {
        match data_type {
            sir::PrimitiveDataType::I64 => self.context.i64_type().as_basic_type_enum(),
        }
    }

    pub fn build(self) -> Module<'ctx> {
        self.module
    }
}