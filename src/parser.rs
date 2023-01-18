use nom::{IResult, bytes::complete::tag, character::complete::{multispace0, satisfy}, multi::{many0, separated_list1}, combinator::{opt, recognize}, Parser, AsChar, sequence::{terminated, delimited, tuple, preceded, separated_pair}, InputTakeAtPosition, error::ParseError};

use crate::sir::{self, DataType};

pub fn module(input: &str) -> IResult<&str, sir::Module> {
    preceded(multispace0, many0(global))
        .map(|globals| sir::Module {
            globals: globals.into_iter().collect(),
        })
        .parse(input)
}

fn global(input: &str) -> IResult<&str, (String, sir::Global)> {
    tuple((
        identifier,
        opt(argument_list),
        preceded(keyword(":"), non_function_type),
        preceded(keyword("="), expression)
    ))
        .map(|(name, arguments, return_type, body)|
            (name, sir::Global {
                arguments: arguments.unwrap_or_default(),
                return_type,
                body
            }))
        .parse(input)
}

fn identifier(input: &str) -> IResult<&str, String> {
    let first_char = satisfy(|c| c.is_lowercase() || c == '_');
    let rest_char = satisfy(|c| c.is_lowercase() || c.is_dec_digit() || c == '_');
    let identifier_str = recognize(first_char.and(many0(rest_char)));
    let identifier = identifier_str
        .map(|id: &str| id.to_string());
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

fn data_type(input: &str) -> IResult<&str, DataType> {
    function_type.or(non_function_type).parse(input)
}

fn function_type(input: &str) -> IResult<&str, DataType> {
    let arguments = separated_list1(keyword(","), data_type);
    let arguments = delimited(keyword("("), arguments, keyword(")"));
    separated_pair(arguments, keyword(":"), non_function_type)
        .map(|(argument_types, return_type)| sir::DataType::Primitive(sir::PrimitiveDataType::Function { argument_types, return_type: Box::new(return_type) }))
        .parse(input)
}

fn non_function_type(input: &str) -> IResult<&str, DataType> {
    keyword("I64").map(|_| sir::DataType::Primitive(sir::PrimitiveDataType::I64)).parse(input)
}

fn type_qualifier(input: &str) -> IResult<&str, DataType> {
    preceded(keyword(":"), data_type).parse(input)
}

fn argument_list(input: &str) -> IResult<&str, Vec<(String, sir::DataType)>> {
    let argument = identifier.and(type_qualifier);
    let arguments = separated_list1(keyword(","), argument);
    delimited(keyword("("), arguments, keyword(")")).parse(input)
}

fn i64_literal(input: &str) -> IResult<&str, sir::Expression> {
    terminated(nom::character::complete::i64, keyword("i64"))
        .map(|val| sir::Expression::I64Literal(val))
        .parse(input)
}

fn expression(input: &str) -> IResult<&str, sir::Expression> {
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

fn add_expression(input: &str) -> IResult<&str, sir::Expression> {
    binary_operation(call_or_member_access,
        |left| preceded(keyword("+"), call_or_member_access)
            .map(move |right| sir::Expression::BinaryOperation {
                operation: sir::BinaryOperation::Add,
                left: Box::new(left.clone()),
                right: Box::new(right)
            }))
    .parse(input)
}

fn call_or_member_access(input: &str) -> IResult<&str, sir::Expression> {
    binary_operation(atom,
        |left| call(left.clone()).or(member_access(left)))
        .parse(input)
}

fn member_access(left: sir::Expression) -> impl FnMut(&str) -> IResult<&str, sir::Expression> {
    move |input| {
        preceded(keyword("."), identifier)
            .map(|member| sir::Expression::MemberAccess {
                left: Box::new(left.clone()),
                member,
            }).parse(input)
    }
}

fn call(function: sir::Expression) -> impl FnMut(&str) -> IResult<&str, sir::Expression> {
    move |input| {
        let arguments = separated_list1(keyword(","), expression);
        delimited(keyword("("), arguments, keyword(")"))
            .map(|arguments| sir::Expression::Call {
                function: Box::new(function.clone()),
                arguments,
            }).parse(input)
    }
}

fn atom(input: &str) -> IResult<&str, sir::Expression> {
    parens
        .or(block)
        .or(reference)
        .or(i64_literal)
        .parse(input)
}

fn parens(input: &str) -> IResult<&str, sir::Expression> {
    delimited(keyword("("), expression, keyword(")")).parse(input)
}

fn reference(input: &str) -> IResult<&str, sir::Expression> {
    identifier.map(|name| sir::Expression::Reference { name }).parse(input)
}

fn block(input: &str) -> IResult<&str, sir::Expression> {
    let scope = terminated(tuple((
        identifier,
        opt(argument_list),
        preceded(keyword("="), expression)
    )), keyword(";"))
    .map(|(name, arguments, body)| match arguments {
        Some(a) => (name.to_string(), sir::Expression::Lambda {
            arguments: a,
            body: Box::new(body),
        }),
        None => (name.to_string(), body)
    });

    let contents = many0(scope).and(expression)
        .map(|(scopes, body)| scopes.into_iter()
            .rev()
            .fold(body, |b, s| sir::Expression::Scope {
                name: s.0,
                value: Box::new(s.1),
                body: Box::new(b)
            })
        );
    delimited(keyword("{"), contents, keyword("}")).parse(input)
}
