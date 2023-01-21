use std::collections::HashMap;

use inkwell::{context::Context, module::Module, builder::Builder, values::{BasicValue, BasicValueEnum, FunctionValue, CallableValue, PointerValue}, types::{BasicTypeEnum, BasicType}, AddressSpace};

use crate::sir;

pub struct Generator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    globals: HashMap<String, FunctionValue<'ctx>>,
    current_function: Option<FunctionValue<'ctx>>,
}

impl <'ctx> Generator<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            module: context.create_module("scrap"),
            builder: context.create_builder(),
            globals: HashMap::new(),
            current_function: None,
        }
    }

    pub fn declare_global_constant(&mut self, name: String, data_type: &sir::DataType) {
        let func_type = match data_type {
            sir::DataType::Primitive(t) => self.primitive_type_to_llvm(t).fn_type(&[], false),
            t => self.context.void_type().fn_type(&[self.type_to_llvm_reference(t).into()], false),
        };
        let func = self.module.add_function(&name, func_type, None);
        self.globals.insert(name, func);
    }

    pub fn write_global_primitive_constant(&mut self, name: &str, value: &sir::Expression) {
        let func = self.globals.get(name).unwrap().clone();

        let entry_block = self.context.append_basic_block(func, "entry");

        self.builder.position_at_end(entry_block);
        let result = self.write_primitive_expression(value);
        self.builder.build_return(Some(&result));
    }

    pub fn write_global_nonprimitive_constant(&mut self, name: &str, value: &sir::Expression) {
        let func = self.globals.get(name).unwrap().clone();

        let entry_block = self.context.append_basic_block(func, "entry");

        self.builder.position_at_end(entry_block);
        self.write_nonprimitive_expression(value, func.get_nth_param(0).unwrap().into_pointer_value());
        self.builder.build_return(None);
    }

    pub fn declare_global_function(&mut self, name: String, arguments: &[(String, sir::DataType)], return_type: &sir::DataType) {
        let mut param_types: Vec<_> = arguments.iter()
            .map(|(_, data_type)| {
                self.type_to_llvm_reference(data_type).into()
            })
            .collect();

        let func_type = match return_type {
            sir::DataType::Primitive(t) => self.primitive_type_to_llvm(t).fn_type(&param_types, false),
            t => {
                param_types.push(self.type_to_llvm_reference(t).into());
                self.context.void_type().fn_type(&param_types, false)
            },
        };

        let func = self.module.add_function(&name, func_type, None);
        self.globals.insert(name, func);
    }

    pub fn write_global_function(&mut self, name: &str, value: &sir::Expression) {
        let func = self.globals.get(name).unwrap().clone();

        let entry_block = self.context.append_basic_block(func, "entry");

        self.current_function = Some(func);

        self.builder.position_at_end(entry_block);
        let result = self.write_primitive_expression(value);
        self.builder.build_return(Some(&result));

        self.current_function = None;
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
            sir::Expression::Call { function, arguments } => {
                let function: CallableValue<'ctx> = self.write_primitive_expression(function).into_pointer_value().try_into().unwrap();
                let args: Vec<_> = arguments.iter()
                    .map(|arg| self.write_primitive_expression(arg).into())
                    .collect();
                self.builder.build_call(function, &args, "").try_as_basic_value().unwrap_left()
            }
            sir::Expression::FunctionParam { index, .. } => self.current_function.unwrap().get_nth_param(*index).unwrap(),
            sir::Expression::GlobalReference { name, data_type: sir::DataType::Primitive(sir::PrimitiveDataType::Function { .. }) } =>
                self.globals.get(name).unwrap().as_global_value().as_pointer_value().as_basic_value_enum(),
            sir::Expression::GlobalReference { name, .. } =>
                self.builder.build_call(self.globals.get(name).unwrap().clone(), &[], "").try_as_basic_value().unwrap_left(),
            _ => todo!(),
        }
    }

    fn write_nonprimitive_expression(&mut self, expr: &sir::Expression, out: PointerValue<'ctx>) {
        match expr {
            sir::Expression::Tuple{ values } => {
                for (i, value) in values.iter().enumerate() {
                    let dest = self.builder.build_struct_gep(out, i as u32, "").unwrap();
                    self.write_nonprimitive_expression(value, dest);
                }
            }
            sir::Expression::GlobalReference { data_type: sir::DataType::Primitive(sir::PrimitiveDataType::Function { .. }), .. } => {
                let value = self.write_primitive_expression(expr);
                self.builder.build_store(out, value);
            }
            sir::Expression::GlobalReference { name, .. } => {
                self.builder.build_call(self.globals.get(name).unwrap().clone(), &[out.into()], "");
            }
            e => {
                let value = self.write_primitive_expression(e);
                self.builder.build_store(out, value);
            },
        }
    }

    fn type_to_llvm(&self, data_type: &sir::DataType) -> BasicTypeEnum<'ctx> {
        match data_type {
            sir::DataType::Primitive(t) => self.primitive_type_to_llvm(t),
            sir::DataType::Tuple(members) => {
                let field_types: Vec<_> = members.iter()
                    .map(|t| self.type_to_llvm(t))
                    .collect();
                self.context.struct_type(&field_types, false).as_basic_type_enum()
            }
        }
    }

    fn primitive_type_to_llvm(&self, data_type: &sir::PrimitiveDataType) -> BasicTypeEnum<'ctx> {
        match data_type {
            sir::PrimitiveDataType::Function { argument_types, return_type } => {
                let mut param_types: Vec<_> = argument_types.iter()
                    .map(|argument_type| self.type_to_llvm_reference(argument_type).into())
                    .collect();
                match return_type.as_ref() {
                    sir::DataType::Primitive(return_type) => {
                        let return_type = self.primitive_type_to_llvm(return_type);
                        return_type.fn_type(&param_types, false).ptr_type(AddressSpace::default()).as_basic_type_enum()
                    },
                    t => {
                        param_types.push(self.type_to_llvm_reference(t).into());
                        self.context.void_type().fn_type(&param_types, false).ptr_type(AddressSpace::default()).as_basic_type_enum()
                    }
                }
            }
            sir::PrimitiveDataType::I64 => self.context.i64_type().as_basic_type_enum(),
        }
    }

    fn type_to_llvm_reference(&self, data_type: &sir::DataType) -> BasicTypeEnum<'ctx> {
        match data_type {
            sir::DataType::Primitive(t) => self.primitive_type_to_llvm(t),
            t => self.type_to_llvm(t).ptr_type(AddressSpace::default()).as_basic_type_enum()
        }
    }

    pub fn build(self) -> Module<'ctx> {
        self.module
    }
}
