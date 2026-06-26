use std::collections::VecDeque;
use crate::calculator::token::Token;
use crate::calculator::ast::{AST, Operator, Function, ProbFunction};
use crate::calculator::calculator::CalculatorError;

/// Verifica si los paréntesis y corchetes están balanceados en una secuencia de tokens.
///
/// Esta función recorre los tokens utilizando una pila para controlar la apertura y cierre
/// de paréntesis `()` y corchetes `[]`. Detecta tanto tokens de cierre sobrantes como
/// aperturas sin cerrar, y también detecta cierres con el tipo incorrecto (ej. `[1 + 2)`).
fn check_parentheses(tokens: &[Token]) -> Result<(), CalculatorError> {
    let mut stack: VecDeque<Token> = VecDeque::new();

    for token in tokens {
        match token {
            Token::LeftParenthesis => {
                stack.push_back(Token::LeftParenthesis);
                if stack.len() > 500 {
                    return Err(CalculatorError::InvalidExpression);
                }
            }
            Token::LeftBracket => {
                stack.push_back(Token::LeftBracket);
                if stack.len() > 500 {
                    return Err(CalculatorError::InvalidExpression);
                }
            }
            Token::RightParenthesis => {
                match stack.pop_back() {
                    Some(Token::LeftParenthesis) => {},
                    _ => return Err(CalculatorError::UnmatchedRightParenthesis),
                }
            },
            Token::RightBracket => {
                match stack.pop_back() {
                    Some(Token::LeftBracket) => {},
                    _ => return Err(CalculatorError::UnmatchedRightParenthesis),
                }
            },
            _ => (),
        }
    }

    if !stack.is_empty() {
        return Err(CalculatorError::UnmatchedLeftParenthesis);
    }

    Ok(())
}

/// Analiza una secuencia de tokens y construye un Árbol de Sintaxis Abstracta (AST).
///
/// Esta función primero valida los paréntesis y luego intenta estructurar jerárquicamente la expresión.
pub(crate) fn parse(tokens: &[Token]) -> Result<(AST, &[Token]), CalculatorError> {
    check_parentheses(tokens)?;
    let (ast, rest) = parse_expression(tokens)?;

    if !rest.is_empty() {
        return Err(CalculatorError::ExtraTokensDetected);
    }

    Ok((ast, rest))
}

/// Analiza una expresión que consiste en términos sumados o restados.
fn parse_expression(tokens: &[Token]) -> Result<(AST, &[Token]), CalculatorError> {

    let (mut lhs, mut rest) = parse_term(tokens)?;
    while let Some(token) = rest.first() {
        match token {
            Token::Plus | Token::Minus => {
                let (rhs, next_tokens) = parse_term(&rest[1..])?;
                lhs = AST::BinOp(
                    Box::new(lhs),
                    if matches!(token, Token::Plus) { Operator::Add } else { Operator::Sub },
                    Box::new(rhs),
                );

                rest = next_tokens;
            },
            _ => break,
        }
    }

    Ok((lhs, rest))
}

/// Analiza un término que consiste en factores multiplicados, divididos, de módulo o porcentajes.
fn parse_term(tokens: &[Token]) -> Result<(AST, &[Token]), CalculatorError> {
    let (mut lhs, mut rest) = parse_unary(tokens)?;
    while let Some(token) = rest.first() {
        match token {
            Token::Multiply | Token::Divide | Token::Mod | Token::Percent => {
                let (rhs, next_tokens) = parse_unary(&rest[1..])?;
                let op = match token {
                    Token::Multiply => Operator::Mul,
                    Token::Divide => Operator::Div,
                    Token::Mod => Operator::Mod,
                    Token::Percent => Operator::Percent,
                    _ => unreachable!(),
                };
                lhs = AST::BinOp(Box::new(lhs), op, Box::new(rhs));
                rest = next_tokens;
            },
            _ => break,
        }
    }
    Ok((lhs, rest))
}

/// Analiza operadores unarios (+ y -).
fn parse_unary(tokens: &[Token]) -> Result<(AST, &[Token]), CalculatorError> {
    match tokens.first() {
        Some(Token::Minus) => {
            let (rhs, rest) = parse_unary(&tokens[1..])?;
            let lhs = AST::Num(-1.0);
            Ok((AST::BinOp(Box::new(lhs), Operator::Mul, Box::new(rhs)), rest))
        }
        Some(Token::Plus) => {
            parse_unary(&tokens[1..])
        }
        _ => parse_power(tokens),
    }
}

/// Analiza una potencia (ej. x ^ y).
fn parse_power(tokens: &[Token]) -> Result<(AST, &[Token]), CalculatorError> {
    let (mut lhs, mut rest) = parse_factor(tokens)?;
    while let Some(token) = rest.first() {
        match token {
            Token::Power => {
                let (rhs, next_tokens) = parse_unary(&rest[1..])?;
                lhs = AST::BinOp(
                    Box::new(lhs),
                    Operator::Power,
                    Box::new(rhs),
                );
                rest = next_tokens;
            },
            _ => break,
        }
    }
    Ok((lhs, rest))
}

fn parse_arguments(tokens: &[Token]) -> Result<(Vec<AST>, &[Token]), CalculatorError> {
    if tokens.first() != Some(&Token::LeftParenthesis) {
        return Err(CalculatorError::InvalidExpression);
    }
    let mut rest = &tokens[1..];
    let mut args = Vec::new();
    if rest.first() == Some(&Token::RightParenthesis) {
        return Ok((args, &rest[1..]));
    }
    loop {
        let (expr, next_rest) = parse_expression(rest)?;
        args.push(expr);
        match next_rest.first() {
            Some(&Token::Comma) => {
                rest = &next_rest[1..];
            }
            Some(&Token::RightParenthesis) => {
                return Ok((args, &next_rest[1..]));
            }
            _ => return Err(CalculatorError::InvalidExpression),
        }
    }
}

fn parse_matrix_literal(tokens: &[Token]) -> Result<(AST, &[Token]), CalculatorError> {
    if tokens.first() != Some(&Token::LeftBracket) {
        return Err(CalculatorError::InvalidExpression);
    }
    let mut rest = &tokens[1..];
    let mut rows = Vec::new();
    let mut current_row = Vec::new();

    if rest.first() == Some(&Token::RightBracket) {
        return Ok((AST::MatrixLiteral(rows), &rest[1..]));
    }

    loop {
        let (expr, next_rest) = parse_expression(rest)?;
        current_row.push(expr);
        match next_rest.first() {
            Some(&Token::Comma) => {
                rest = &next_rest[1..];
            }
            Some(&Token::Semicolon) => {
                rows.push(current_row);
                current_row = Vec::new();
                rest = &next_rest[1..];
            }
            Some(&Token::RightBracket) => {
                if !current_row.is_empty() {
                    rows.push(current_row);
                }
                if !rows.is_empty() {
                    let first_len = rows[0].len();
                    for r in &rows {
                        if r.len() != first_len {
                            return Err(CalculatorError::InvalidExpression);
                        }
                    }
                }
                return Ok((AST::MatrixLiteral(rows), &next_rest[1..]));
            }
            _ => return Err(CalculatorError::InvalidExpression),
        }
    }
}

/// Analiza un factor básico: números, variables, paréntesis o llamadas a funciones científicas.
fn parse_factor(tokens: &[Token]) -> Result<(AST, &[Token]), CalculatorError> {
    let (mut ast, mut rest) = match tokens.first() {
        Some(Token::LeftBracket) => {
            parse_matrix_literal(tokens)
        }
        Some(Token::Deriv) | Some(Token::Intg) | Some(Token::Sum) | Some(Token::Prod) => {
            let func_token = tokens.first().unwrap();
            let (args, rest) = parse_arguments(&tokens[1..])?;
            let ast = match func_token {
                Token::Deriv => {
                    if args.len() == 2 {
                        AST::Deriv(Box::new(args[0].clone()), "x".to_string(), Box::new(args[1].clone()))
                    } else if args.len() == 3 {
                        let var_name = match &args[1] {
                            AST::Var(name) => name.clone(),
                            _ => return Err(CalculatorError::InvalidExpression),
                        };
                        AST::Deriv(Box::new(args[0].clone()), var_name, Box::new(args[2].clone()))
                    } else {
                        return Err(CalculatorError::InvalidExpression);
                    }
                }
                Token::Intg => {
                    if args.len() == 3 {
                        AST::Intg(Box::new(args[0].clone()), "x".to_string(), Box::new(args[1].clone()), Box::new(args[2].clone()))
                    } else if args.len() == 4 {
                        let var_name = match &args[1] {
                            AST::Var(name) => name.clone(),
                            _ => return Err(CalculatorError::InvalidExpression),
                        };
                        AST::Intg(Box::new(args[0].clone()), var_name, Box::new(args[2].clone()), Box::new(args[3].clone()))
                    } else {
                        return Err(CalculatorError::InvalidExpression);
                    }
                }
                Token::Sum => {
                    if args.len() == 1 {
                        // sum([1,2,3]) = suma de vector
                        AST::Func(Function::Sum, Box::new(args[0].clone()))
                    } else if args.len() == 4 {
                        let var_name = match &args[1] {
                            AST::Var(name) => name.clone(),
                            _ => return Err(CalculatorError::InvalidExpression),
                        };
                        AST::Sum(Box::new(args[0].clone()), var_name, Box::new(args[2].clone()), Box::new(args[3].clone()))
                    } else {
                        return Err(CalculatorError::InvalidExpression);
                    }
                }
                Token::Prod => {
                    if args.len() != 4 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let var_name = match &args[1] {
                        AST::Var(name) => name.clone(),
                        _ => return Err(CalculatorError::InvalidExpression),
                    };
                    AST::Prod(Box::new(args[0].clone()), var_name, Box::new(args[2].clone()), Box::new(args[3].clone()))
                }
                _ => unreachable!(),
            };
            Ok((ast, rest))
        }
        Some(Token::Number(n)) => Ok((AST::Num(*n), &tokens[1..])),
        Some(Token::Percent) => {
            // El signo '%' al principio de un factor como "%50" equivale a multiplicar por 0.01
            Ok((AST::Num(0.01), &tokens[1..]))
        },
        Some(Token::Variable(name)) => Ok((AST::Var(name.clone()), &tokens[1..])),
        Some(Token::LeftParenthesis) => {
            let (expr, rest) = parse_expression(&tokens[1..])?;
            match rest.first() {
                Some(Token::RightParenthesis) => Ok((expr, &rest[1..])),
                _ => Err(CalculatorError::InvalidExpression),
            }
        },
        Some(Token::Log) => {
            let (args, rest) = parse_arguments(&tokens[1..])?;
            if args.len() == 1 {
                Ok((AST::Func(Function::Log, Box::new(args[0].clone())), rest))
            } else if args.len() == 2 {
                Ok((AST::Func2(crate::calculator::ast::Function2::LogBase, Box::new(args[0].clone()), Box::new(args[1].clone())), rest))
            } else {
                Err(CalculatorError::InvalidExpression)
            }
        },
        Some(Token::Sin) | Some(Token::Cos) | Some(Token::Ctan) | Some(Token::Tan) | Some(Token::Ln) | Some(Token::Sqrt) | Some(Token::Abs) | Some(Token::Asin) | Some(Token::Acos) | Some(Token::Atan) | Some(Token::Sinh) | Some(Token::Cosh) | Some(Token::Tanh) | Some(Token::Asinh) | Some(Token::Acosh) | Some(Token::Atanh) | Some(Token::Fact) | Some(Token::Floor) | Some(Token::Ceil) | Some(Token::Round) | Some(Token::Trunc) | Some(Token::Int) | Some(Token::Fract) | Some(Token::Cbrt) | Some(Token::Re) | Some(Token::Im) | Some(Token::Conj) | Some(Token::Arg) | Some(Token::Exp) | Some(Token::Mean) | Some(Token::Median) | Some(Token::VarFunc) | Some(Token::Std) | Some(Token::Det) | Some(Token::Inv) | Some(Token::Transpose) | Some(Token::Sort) | Some(Token::Tr) => {
            let func_token = tokens.first().unwrap();
            if tokens.get(1) != Some(&Token::LeftParenthesis) {
                return Err(CalculatorError::InvalidExpression);
            }
            let (expr, rest) = parse_expression(&tokens[2..])?;
            if rest.get(0) != Some(&Token::RightParenthesis) {
                return Err(CalculatorError::InvalidExpression);
            }
            let func = match func_token {
                Token::Sin => Function::Sin,
                Token::Cos => Function::Cos,
                Token::Tan => Function::Tan,
                Token::Ln => Function::Ln,
                Token::Ctan => Function::Ctan,
                Token::Sqrt => Function::Sqrt,
                Token::Abs => Function::Abs,
                Token::Asin => Function::Asin,
                Token::Acos => Function::Acos,
                Token::Atan => Function::Atan,
                Token::Sinh => Function::Sinh,
                Token::Cosh => Function::Cosh,
                Token::Tanh => Function::Tanh,
                Token::Asinh => Function::Asinh,
                Token::Acosh => Function::Acosh,
                Token::Atanh => Function::Atanh,
                Token::Fact => Function::Fact,
                Token::Floor => Function::Floor,
                Token::Ceil => Function::Ceil,
                Token::Round => Function::Round,
                Token::Trunc => Function::Trunc,
                Token::Int => Function::Int,
                Token::Fract => Function::Fract,
                Token::Cbrt => Function::Cbrt,
                Token::Re => Function::Re,
                Token::Im => Function::Im,
                Token::Conj => Function::Conj,
                Token::Arg => Function::Arg,
                Token::Exp => Function::Exp,
                Token::Mean => Function::Mean,
                Token::Median => Function::Median,
                Token::VarFunc => Function::Var,
                Token::Std => Function::Std,
                Token::Det => Function::Det,
                Token::Inv => Function::Inv,
                Token::Transpose => Function::Transpose,
                Token::Sort => Function::Sort,
                Token::Tr => Function::Tr,
                _ => return Err(CalculatorError::UnexpectedToken),
            };
            Ok((AST::Func(func, Box::new(expr)), &rest[1..]))
        },
        Some(Token::Min) | Some(Token::Max) => {
            // min/max: 1 arg = operacion sobre vector, 2 args = comparacion entre escalares
            let func_token = tokens.first().unwrap();
            let (args, rest) = parse_arguments(&tokens[1..])?;
            if args.len() == 1 {
                let func = match func_token {
                    Token::Min => Function::MinVec,
                    Token::Max => Function::MaxVec,
                    _ => unreachable!(),
                };
                Ok((AST::Func(func, Box::new(args[0].clone())), rest))
            } else if args.len() == 2 {
                parse_func2(tokens)
            } else {
                Err(CalculatorError::InvalidExpression)
            }
        },
        Some(Token::Mod) | Some(Token::Gcd) | Some(Token::Lcm) | Some(Token::Ncr) | Some(Token::Npr) | Some(Token::Root) | Some(Token::Polar) | Some(Token::Cov) | Some(Token::Corr) | Some(Token::LinReg) => {
            parse_func2(tokens)
        },
        Some(Token::PolyReg) => {
            if tokens.get(1) != Some(&Token::LeftParenthesis) {
                return Err(CalculatorError::InvalidExpression);
            }
            let (args, rest) = parse_arguments(&tokens[1..])?;
            if args.len() != 3 {
                return Err(CalculatorError::InvalidExpression);
            }
            Ok((AST::PolyReg(Box::new(args[0].clone()), Box::new(args[1].clone()), Box::new(args[2].clone())), rest))
        },
        Some(Token::Rand) | Some(Token::NormPdf) | Some(Token::NormCdf) | Some(Token::BinoPdf) | Some(Token::BinoCdf) | Some(Token::PoissPdf) | Some(Token::PoissCdf) => {
            let func_token = tokens.first().unwrap();
            let (args, rest) = parse_arguments(&tokens[1..])?;
            let func = match func_token {
                Token::Rand => ProbFunction::Rand,
                Token::NormPdf => ProbFunction::NormPdf,
                Token::NormCdf => ProbFunction::NormCdf,
                Token::BinoPdf => ProbFunction::BinoPdf,
                Token::BinoCdf => ProbFunction::BinoCdf,
                Token::PoissPdf => ProbFunction::PoissPdf,
                Token::PoissCdf => ProbFunction::PoissCdf,
                _ => unreachable!(),
            };
            Ok((AST::ProbFunc(func, args), rest))
        },
        Some(Token::Pi) => Ok((AST::Const(std::f64::consts::PI), &tokens[1..])),
        Some(Token::E) => Ok((AST::Const(std::f64::consts::E), &tokens[1..])),
        Some(Token::Tau) => Ok((AST::Const(std::f64::consts::TAU), &tokens[1..])),
        Some(Token::Phi) => Ok((AST::Const(1.6180339887498949_f64), &tokens[1..])),
        Some(Token::Sqrt2) => Ok((AST::Const(std::f64::consts::SQRT_2), &tokens[1..])),
        Some(Token::ConstC) => Ok((AST::Const(299792458.0), &tokens[1..])),
        Some(Token::ConstH) => Ok((AST::Const(6.62607015e-34), &tokens[1..])),
        Some(Token::ConstG) => Ok((AST::Const(6.6743e-11), &tokens[1..])),
        Some(Token::Ans) => Ok((AST::Var("__ANS__".to_string()), &tokens[1..])),
        Some(Token::LogBase(base)) => {
            if tokens.get(1) != Some(&Token::LeftParenthesis) {
                return Err(CalculatorError::InvalidExpression);
            }
            let (expr, rest) = parse_expression(&tokens[2..])?;
            match rest.first() {
                Some(Token::RightParenthesis) => Ok((AST::LogBase(*base, Box::new(expr)), &rest[1..])),
                _ => Err(CalculatorError::InvalidExpression),
            }
        },

        _ => Err(CalculatorError::UnexpectedToken),
    }?;

    while let Some(Token::Excl) = rest.first() {
        ast = AST::Func(Function::Fact, Box::new(ast));
        rest = &rest[1..];
    }

    Ok((ast, rest))
}

/// Analiza una llamada a una función de múltiples argumentos genérica: `func(arg1, arg2, ...)`.
///
/// Usa `parse_arguments` internamente para extraer N argumentos separados por comas.
/// Valida que el token de la función corresponda a una función de exactamente 2 argumentos.
fn parse_func2(tokens: &[Token]) -> Result<(AST, &[Token]), CalculatorError> {
    let func_token = tokens.first().unwrap();
    if tokens.get(1) != Some(&Token::LeftParenthesis) {
        return Err(CalculatorError::InvalidExpression);
    }
    // Usa parse_arguments para extraer todos los argumentos separados por coma
    let (args, rest) = parse_arguments(&tokens[1..])?;
    if args.len() != 2 {
        return Err(CalculatorError::InvalidExpression);
    }
    let func = match func_token {
        Token::Min    => crate::calculator::ast::Function2::Min,
        Token::Max    => crate::calculator::ast::Function2::Max,
        Token::Mod    => crate::calculator::ast::Function2::Mod,
        Token::Gcd    => crate::calculator::ast::Function2::Gcd,
        Token::Lcm    => crate::calculator::ast::Function2::Lcm,
        Token::Ncr    => crate::calculator::ast::Function2::Ncr,
        Token::Npr    => crate::calculator::ast::Function2::Npr,
        Token::Root   => crate::calculator::ast::Function2::Root,
        Token::Polar  => crate::calculator::ast::Function2::Polar,
        Token::Cov    => crate::calculator::ast::Function2::Cov,
        Token::Corr   => crate::calculator::ast::Function2::Corr,
        Token::LinReg => crate::calculator::ast::Function2::LinReg,
        _ => return Err(CalculatorError::UnexpectedToken),
    };
    let (arg1, arg2) = (args[0].clone(), args[1].clone());
    Ok((AST::Func2(func, Box::new(arg1), Box::new(arg2)), rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_number() {
        let tokens = vec![Token::Number(42.0)];
        let (ast, _) = parse(&tokens).unwrap();
        assert_eq!(ast, AST::Num(42.0));
    }

    #[test]
    fn parse_simple_addition() {
        let tokens = vec![Token::Number(1.0), Token::Plus, Token::Number(2.0)];
        let (ast, _) = parse(&tokens).unwrap();
        match ast {
            AST::BinOp(lhs, op, rhs) => {
                assert_eq!(*lhs, AST::Num(1.0));
                assert_eq!(op, Operator::Add);
                assert_eq!(*rhs, AST::Num(2.0));
            },
            _ => panic!("Expected BinOp AST node"),
        }
    }

    #[test]
    fn parse_error_unmatched_left_parenthesis() {
        let tokens = vec![Token::LeftParenthesis, Token::Number(1.0)];
        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn parse_error_unmatched_right_parenthesis() {
        let tokens = vec![Token::Number(1.0), Token::RightParenthesis];
        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn parse_error_unexpected_token() {
        let tokens = vec![Token::Plus];
        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn test_check_parentheses_balanced() {
        let tokens = vec![Token::LeftParenthesis, Token::Number(1.0), Token::RightParenthesis];
        assert_eq!(check_parentheses(&tokens), Ok(()));
    }

    #[test]
    fn test_check_parentheses_unmatched_right() {
        let tokens = vec![Token::RightParenthesis, Token::Number(1.0)];
        assert_eq!(check_parentheses(&tokens), Err(CalculatorError::UnmatchedRightParenthesis));
    }

    #[test]
    fn test_check_parentheses_unmatched_left() {
        let tokens = vec![Token::LeftParenthesis, Token::Number(1.0)];
        assert_eq!(check_parentheses(&tokens), Err(CalculatorError::UnmatchedLeftParenthesis));
    }

    #[test]
    fn test_parse_simple_expression() {
        let tokens = vec![Token::Number(1.0), Token::Plus, Token::Number(2.0)];
        let result = parse(&tokens).unwrap();
        // You might need a custom assert function or macro to compare AST nodes
        assert!(matches!(result.0, AST::BinOp(_, Operator::Add, _)));
        assert!(result.1.is_empty());
    }

    #[test]
    fn test_parse_expression_with_extra_tokens() {
        let tokens = vec![Token::Number(1.0), Token::Plus, Token::Number(2.0), Token::Multiply];
        assert_eq!(parse(&tokens), Err(CalculatorError::UnexpectedToken));
    }

    #[test]
    fn test_parse_term_simple_multiplication() {
        let tokens = vec![Token::Number(2.0), Token::Multiply, Token::Number(3.0)];
        let result = parse_term(&tokens).unwrap();
        assert!(matches!(result.0, AST::BinOp(_, Operator::Mul, _)));
        assert!(result.1.is_empty());
    }

    // MEJ-13: Tests de balance de corchetes []
    #[test]
    fn test_check_brackets_balanced() {
        // [1, 2, 3] — corchetes balanceados
        let tokens = vec![
            Token::LeftBracket, Token::Number(1.0), Token::Comma,
            Token::Number(2.0), Token::RightBracket,
        ];
        assert_eq!(check_parentheses(&tokens), Ok(()));
    }

    #[test]
    fn test_check_brackets_unmatched_right() {
        // 1, 2] — corchete de cierre sin apertura
        let tokens = vec![Token::Number(1.0), Token::Comma, Token::Number(2.0), Token::RightBracket];
        assert_eq!(check_parentheses(&tokens), Err(CalculatorError::UnmatchedRightParenthesis));
    }

    #[test]
    fn test_check_brackets_unmatched_left() {
        // [1, 2 — corchete de apertura sin cierre
        let tokens = vec![Token::LeftBracket, Token::Number(1.0), Token::Comma, Token::Number(2.0)];
        assert_eq!(check_parentheses(&tokens), Err(CalculatorError::UnmatchedLeftParenthesis));
    }

    #[test]
    fn test_check_mixed_paren_bracket_mismatch() {
        // [1 + 2) — abre con [ pero cierra con ) → error
        let tokens = vec![
            Token::LeftBracket, Token::Number(1.0), Token::Plus,
            Token::Number(2.0), Token::RightParenthesis,
        ];
        assert_eq!(check_parentheses(&tokens), Err(CalculatorError::UnmatchedRightParenthesis));
    }
}