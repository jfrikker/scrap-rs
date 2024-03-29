use std::{fmt::Write, mem::{replace, swap}};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum, CallableValue, FunctionValue, PointerValue, BasicMetadataValueEnum},
    AddressSpace, intrinsics::Intrinsic,
};

use crate::sir;

pub struct Generator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

    current_function: Option<FunctionValue<'ctx>>,
}

impl<'ctx> Generator<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            module: context.create_module("scrap"),
            builder: context.create_builder(),
            current_function: None,
        }
    }


    pub fn declare_global_constant(&mut self, name: String, data_type: &sir::DataType) {
        let func_type = match data_type {
            sir::DataType::Primitive(t) => self.primitive_type_to_llvm(t).fn_type(&[], false),
            t => self
                .context
                .void_type()
                .fn_type(&[self.type_to_llvm_reference(t).into()], false),
        };
        self.module.add_function(&name, func_type, None);
    }

    pub fn write_global_primitive_constant(&mut self, name: &str, value: &sir::Expression) {
        let func = self.module.get_function(name).unwrap();

        let entry_block = self.context.append_basic_block(func, "entry");

        self.builder.position_at_end(entry_block);
        let result = self.write_expression(value);
        self.builder.build_return(Some(&result));
    }

    pub fn write_global_nonprimitive_constant(&mut self, name: &str, value: &sir::Expression) {
        let func = self.module.get_function(name).unwrap();

        let entry_block = self.context.append_basic_block(func, "entry");

        self.builder.position_at_end(entry_block);
        self.write_expression_into(
            value,
            func.get_nth_param(0).unwrap().into_pointer_value(),
        );
        self.builder.build_return(None);
    }

    pub fn declare_global_function(
        &mut self,
        name: String,
        arguments: &[(String, sir::DataType)],
        return_type: &sir::DataType,
    ) {
        let mut param_types: Vec<_> = arguments
            .iter()
            .map(|(_, data_type)| self.type_to_llvm_reference(data_type).into())
            .collect();

        let func_type = match return_type {
            sir::DataType::Primitive(t) => {
                self.primitive_type_to_llvm(t).fn_type(&param_types, false)
            }
            t => {
                param_types.push(self.type_to_llvm_reference(t).into());
                self.context.void_type().fn_type(&param_types, false)
            }
        };

        self.module.add_function(&name, func_type, None);
    }

    pub fn write_global_function(&mut self, name: &str, value: &sir::Expression) {
        let func = self.module.get_function(name).unwrap();

        let entry_block = self.context.append_basic_block(func, "entry");

        self.current_function = Some(func);

        self.builder.position_at_end(entry_block);

        eprintln!("{:?}", value.data_type());
        if value.data_type().is_primitive() {
            let result = self.write_expression(value);
            self.builder.build_return(Some(&result));
        } else {
            let out = func.get_last_param().unwrap().into_pointer_value();
            self.write_expression_into(value, out);
            self.builder.build_return(None);
        }

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

    fn write_expression(&mut self, expr: &sir::Expression) -> BasicValueEnum<'ctx> {
        match expr {
            sir::Expression::BinaryOperation {
                operation: sir::BinaryOperation::Add,
                left,
                right,
            } => {
                let left = self
                    .write_expression(left.as_ref())
                    .into_int_value();
                let right = self
                    .write_expression(right.as_ref())
                    .into_int_value();
                self.builder
                    .build_int_add(left, right, "add")
                    .as_basic_value_enum()
            }
            sir::Expression::Call {
                function,
                arguments,
            } => {
                let data_type = expr.data_type();
                let sir::DataType::Primitive(sir::PrimitiveDataType::Function { return_type, .. }) = data_type.as_ref() else {panic!("non function")};
                if return_type.is_primitive() {
                    let function: CallableValue<'ctx> = self
                        .write_expression(function)
                        .into_pointer_value()
                        .try_into()
                        .unwrap();
                    let arguments: Vec<BasicMetadataValueEnum<'ctx>> = arguments.iter()
                        .map(|argument| self.write_expression(argument).into())
                        .collect();

                    self.builder
                        .build_call(function, &arguments, "")
                        .try_as_basic_value()
                        .unwrap_left()
                } else {
                    let temp = self.builder.build_alloca(self.type_to_llvm(return_type.as_ref()), "");
                    self.write_expression_into(expr, temp);
                    temp.as_basic_value_enum()
                }
            }
            sir::Expression::FunctionParam { index, .. } => self
                .current_function
                .unwrap()
                .get_nth_param(*index)
                .unwrap(),
            sir::Expression::GlobalReference {
                name,
                data_type: sir::DataType::Primitive(sir::PrimitiveDataType::Function { .. }),
            } => self
                .module
                .get_function(name)
                .unwrap()
                .as_global_value()
                .as_pointer_value()
                .as_basic_value_enum(),
            sir::Expression::GlobalReference { name, .. } => self
                .builder
                .build_call(self.module.get_function(name).unwrap().clone(), &[], "")
                .try_as_basic_value()
                .unwrap_left(),
            sir::Expression::I64Literal(val) => self
                .context
                .i64_type()
                .const_int(*val as u64, true)
                .as_basic_value_enum(),
            sir::Expression::MemberAccess { left, member } => {
                let data_type = left.data_type();
                let left = self.write_expression(left);
                let index = data_type.field_index(member).unwrap();
                let ptr = self.builder.build_struct_gep(left.into_pointer_value(), index as u32, "").unwrap();
                if data_type.field_type(&member).unwrap().is_primitive() {
                    self.builder.build_load(ptr, "").as_basic_value_enum()
                } else {
                    ptr.as_basic_value_enum()
                }
            }
            e => {
                let data_type = e.data_type();
                let temp = self.builder.build_alloca(self.type_to_llvm(data_type.as_ref()), "");
                self.write_expression_into(e, temp);
                temp.as_basic_value_enum()
            },
        }
    }

    fn write_expression_into(&mut self, expr: &sir::Expression, out: PointerValue<'ctx>) {
        match expr {
            sir::Expression::Call {
                function,
                arguments,
            } => {
                let function: CallableValue<'ctx> = self
                    .write_expression(function)
                    .into_pointer_value()
                    .try_into()
                    .unwrap();
                let mut arguments: Vec<BasicMetadataValueEnum<'ctx>> = arguments.iter()
                    .map(|argument| self.write_expression(argument).into())
                    .collect();
                arguments.push(out.into());
                self.builder.build_call(function, &arguments, "");
            }
            sir::Expression::FunctionParam { index, data_type } => {
                let input = self.current_function.unwrap().get_nth_param(*index).unwrap().into_pointer_value();
                self.write_clone(data_type, input, out);
            }
            sir::Expression::GlobalReference {
                data_type: sir::DataType::Primitive(sir::PrimitiveDataType::Function { .. }),
                ..
            } => {
                let value = self.write_expression(expr);
                self.builder.build_store(out, value);
            }
            sir::Expression::GlobalReference { name, .. } => {
                self.builder
                    .build_call(self.module.get_function(name).unwrap().clone(), &[out.into()], "");
            }
            sir::Expression::Tuple { values } => {
                for (i, value) in values.iter().enumerate() {
                    let dest = self.builder.build_struct_gep(out, i as u32, "").unwrap();
                    self.write_expression_into(value, dest);
                }
            }
            e => {
                let value = self.write_expression(e);
                self.builder.build_store(out, value);
            }
        }
    }

    fn type_to_llvm(&self, data_type: &sir::DataType) -> BasicTypeEnum<'ctx> {
        match data_type {
            sir::DataType::Primitive(t) => self.primitive_type_to_llvm(t),
            sir::DataType::Tuple(members) => {
                let field_types: Vec<_> = members.iter().map(|t| self.type_to_llvm(t)).collect();
                self.context
                    .struct_type(&field_types, false)
                    .as_basic_type_enum()
            }
        }
    }

    fn primitive_type_to_llvm(&self, data_type: &sir::PrimitiveDataType) -> BasicTypeEnum<'ctx> {
        match data_type {
            sir::PrimitiveDataType::Function {
                argument_types,
                return_type,
            } => {
                let mut param_types: Vec<_> = argument_types
                    .iter()
                    .map(|argument_type| self.type_to_llvm_reference(argument_type).into())
                    .collect();
                match return_type.as_ref() {
                    sir::DataType::Primitive(return_type) => {
                        let return_type = self.primitive_type_to_llvm(return_type);
                        return_type
                            .fn_type(&param_types, false)
                            .ptr_type(AddressSpace::default())
                            .as_basic_type_enum()
                    }
                    t => {
                        param_types.push(self.type_to_llvm_reference(t).into());
                        self.context
                            .void_type()
                            .fn_type(&param_types, false)
                            .ptr_type(AddressSpace::default())
                            .as_basic_type_enum()
                    }
                }
            }
            sir::PrimitiveDataType::I64 => self.context.i64_type().as_basic_type_enum(),
        }
    }

    fn type_to_llvm_reference(&self, data_type: &sir::DataType) -> BasicTypeEnum<'ctx> {
        match data_type {
            sir::DataType::Primitive(t) => self.primitive_type_to_llvm(t),
            t => self
                .type_to_llvm(t)
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum(),
        }
    }

    fn write_clone(&mut self, data_type: &sir::DataType, input: PointerValue<'ctx>, out: PointerValue<'ctx>) {
        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default()).into();
        let i64_type = self.context.i64_type().into();
        let i1_type = self.context.bool_type().into();
        let size = self.type_to_llvm(data_type).size_of().unwrap();
        let memcpy = Intrinsic::find("llvm.memcpy.p0i8.p0i8.i64").unwrap().get_declaration(&self.module, &[i8_ptr_type, i8_ptr_type, i64_type, i1_type]).unwrap();
        let input = self.builder.build_bitcast(input, i8_ptr_type, "");
        let out = self.builder.build_bitcast(out , i8_ptr_type, "");
        self.builder.build_call(memcpy, &[out.into(), input.into(), size.into(), self.context.bool_type().const_zero().into()], "");
    }

    pub fn build(self) -> Module<'ctx> {
        self.module
    }
}
