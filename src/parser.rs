use std::rc::Rc;

use nom::{IResult, bytes::complete::tag, character::complete::{multispace0, satisfy}, multi::{many0, separated_list1}, combinator::{opt, recognize}, Parser, AsChar, sequence::{terminated, delimited, tuple, preceded, separated_pair}, InputTakeAtPosition, error::ParseError};

use crate::sir::{self, DataType};

pub fn module(input: &str) -> IResult<&str, Rc<sir::Module>> {
    preceded(multispace0, many0(global))
        .map(|globals| Rc::new(sir::Module {
            globals: globals.into_iter().collect(),
        }))
        .parse(input)
}

fn global(input: &str) -> IResult<&str, (Rc<String>, Rc<sir::Global>)> {
    tuple((
        identifier,
        opt(argument_list),
        preceded(keyword(":"), non_function_type),
        preceded(keyword("="), expression)
    ))
        .map(|(name, arguments, return_type, body)|
            (name, Rc::new(sir::Global {
                arguments: arguments.unwrap_or_default(),
                return_type,
                body
            })))
        .parse(input)
}

fn identifier(input: &str) -> IResult<&str, Rc<String>> {
    let first_char = satisfy(|c| c.is_lowercase() || c == '_');
    let rest_char = satisfy(|c| c.is_lowercase() || c.is_dec_digit() || c == '_');
    let identifier_str = recognize(first_char.and(many0(rest_char)));
    let identifier = identifier_str
        .map(|id: &str| Rc::new(id.to_string()));
    ws_terminated(identifier).parse(input)
}

fn ws_terminated<I, O, E>(parser: impl Parser<I, O, E>) -> impl FnMut(I) -> IResult<I, O, E>
where
  I: InputTakeAtPosition,
  <I as InputTakeAtPosition>::Item: AsChar + Clone,
  E: ParseError<I>,
{
    terminated(parser, multispace0)
}

fn keyword(word: &str) -> impl Parser<&str, &str, nom::error::Error<&str>> {
    ws_terminated(tag(word))
}

fn data_type(input: &str) -> IResult<&str, Rc<DataType>> {
    function_type.or(non_function_type).parse(input)
}

fn function_type(input: &str) -> IResult<&str, Rc<DataType>> {
    let arguments = separated_list1(keyword(","), data_type);
    let arguments = delimited(keyword("("), arguments, keyword(")"));
    separated_pair(arguments, keyword(":"), non_function_type)
        .map(|(argument_types, return_type)| Rc::new(sir::DataType::Function { argument_types, return_type }))
        .parse(input)
}

fn non_function_type(input: &str) -> IResult<&str, Rc<DataType>> {
    keyword("I64").map(|_| Rc::new(sir::DataType::I64)).parse(input)
}

fn type_qualifier(input: &str) -> IResult<&str, Rc<DataType>> {
    preceded(keyword(":"), data_type).parse(input)
}

fn argument_list(input: &str) -> IResult<&str, Vec<(Rc<String>, Rc<sir::DataType>)>> {
    let argument = identifier.and(type_qualifier);
    let arguments = separated_list1(keyword(","), argument);
    delimited(keyword("("), arguments, keyword(")")).parse(input)
}

fn i64_literal(input: &str) -> IResult<&str, Rc<sir::Expression>> {
    terminated(nom::character::complete::i64, keyword("i64"))
        .map(|val| Rc::new(sir::Expression::I64Literal(val)))
        .parse(input)
}

fn expression(input: &str) -> IResult<&str, Rc<sir::Expression>> {
    add_expression.parse(input)
}

fn binary_operation<I: Clone, O: Clone + 'static, E: ParseError<I>, R: Parser<I, O, E>>(mut first: impl Parser<I, O, E>, rest: impl Fn(O) -> R) -> impl FnMut(I) -> IResult<I, O, E> {
    move |input| {
        let (input, f) = first.parse(input)?;
        binary_operation_step(f, &rest).parse(input)
    }
}

fn binary_operation_step<'r, I: Clone, O: Clone + 'static, E: ParseError<I>, R: Parser<I, O, E>>(left: O, rest: &'r impl Fn(O) -> R) -> impl FnMut(I) -> IResult<I, O, E> + 'r {
    move |input| {
        let (input, next) = opt(rest(left.clone())).parse(input)?;
        match next {
            Some(n) => binary_operation_step(n, rest).parse(input),
            None => Ok((input, left.clone()))
        }
    }
}

fn add_expression(input: &str) -> IResult<&str, Rc<sir::Expression>> {
    binary_operation(call_or_member_access,
        |left| preceded(keyword("+"), call_or_member_access)
            .map(move |right| Rc::new(sir::Expression::BinaryOperation {
                operation: sir::BinaryOperation::Add,
                left: left.clone(),
                right
            })))
    .parse(input)
}

fn call_or_member_access(input: &str) -> IResult<&str, Rc<sir::Expression>> {
    binary_operation(atom,
        |left| call(left.clone()).or(member_access(left)))
        .parse(input)
}

fn member_access(left: Rc<sir::Expression>) -> impl FnMut(&str) -> IResult<&str, Rc<sir::Expression>> {
    move |input| {
        preceded(keyword("."), identifier)
            .map(|member| Rc::new(sir::Expression::MemberAccess {
                left: left.clone(),
                member,
            })).parse(input)
    }
}

fn call(function: Rc<sir::Expression>) -> impl FnMut(&str) -> IResult<&str, Rc<sir::Expression>> {
    move |input| {
        let arguments = separated_list1(keyword(","), expression);
        delimited(keyword("("), arguments, keyword(")"))
            .map(|arguments| Rc::new(sir::Expression::Call {
                function: function.clone(),
                arguments,
            })).parse(input)
    }
}

fn atom(input: &str) -> IResult<&str, Rc<sir::Expression>> {
    parens
        .or(block)
        .or(reference)
        .or(i64_literal)
        .parse(input)
}

fn parens(input: &str) -> IResult<&str, Rc<sir::Expression>> {
    delimited(keyword("("), expression, keyword(")")).parse(input)
}

fn reference(input: &str) -> IResult<&str, Rc<sir::Expression>> {
    identifier.map(|name| Rc::new(sir::Expression::Reference { name })).parse(input)
}

fn block(input: &str) -> IResult<&str, Rc<sir::Expression>> {
    let scope = terminated(tuple((
        identifier,
        opt(argument_list),
        preceded(keyword("="), expression)
    )), keyword(";"))
    .map(|(name, arguments, body)| match arguments {
        Some(a) => (Rc::new(name.to_string()), Rc::new(sir::Expression::Lambda {
            arguments: a,
            body,
        })),
        None => (Rc::new(name.to_string()), body)
    });

    let contents = many0(scope).and(expression)
        .map(|(scopes, body)| scopes.into_iter()
            .rev()
            .fold(body, |b, s| Rc::new(sir::Expression::Scope {
                name: s.0,
                value: s.1,
                body: b
            }))
        );
    delimited(keyword("{"), contents, keyword("}")).parse(input)
}