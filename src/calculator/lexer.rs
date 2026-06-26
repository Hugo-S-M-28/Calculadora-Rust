#![allow(clippy::approx_constant)]
use crate::calculator::calculator::CalculatorError;
use crate::calculator::token::Token;

/// Realiza el análisis léxico de una expresión matemática, convirtiéndola en una secuencia de tokens.
///
/// Esta función procesa secuencialmente cada carácter de la cadena de entrada, categorizándolos en tokens
/// que representan números, operadores aritméticos, paréntesis e identificadores de variables, funciones o constantes.
///
/// # Argumentos
///
/// * `input` - Una porción de cadena que contiene la expresión matemática a tokenizar.
///
/// # Retorno
///
/// * `Ok(Vec<Token>)` - Un vector con los tokens obtenidos de la expresión de entrada.
/// * `Err(CalculatorError)` - Un error si el analizador léxico encuentra un patrón no reconocido o sintaxis inválida.
pub(crate) fn lex(input: &str) -> Result<Vec<Token>, CalculatorError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '0'..='9' | '.' => {
                // Analiza el número
                let number = parse_number(&mut chars)?;
                // Maneja combinaciones donde una constante sigue inmediatamente a un número (ej. 2pi -> 2 * pi)
                handle_number_constant_combination(&mut chars, &mut tokens, number);
            },
            '+' => { chars.next(); tokens.push(Token::Plus); },
            '-' => { chars.next(); tokens.push(Token::Minus); },
            '*' => {
                chars.next();
                if chars.peek() == Some(&'*') {
                    chars.next();
                    tokens.push(Token::Power);
                } else {
                    tokens.push(Token::Multiply);
                }
            },
            '/' => { chars.next(); tokens.push(Token::Divide); },
            '^' => { chars.next(); tokens.push(Token::Power); },
            '%' => { chars.next(); tokens.push(Token::Percent); },
            ',' => { chars.next(); tokens.push(Token::Comma); },
            '[' => { chars.next(); tokens.push(Token::LeftBracket); },
            ']' => { chars.next(); tokens.push(Token::RightBracket); },
            ';' => { chars.next(); tokens.push(Token::Semicolon); },
            '(' => { chars.next(); tokens.push(Token::LeftParenthesis); },
            ')' => { chars.next(); tokens.push(Token::RightParenthesis); },
            '=' => { chars.next(); tokens.push(Token::Equal); },
            '!' => { chars.next(); tokens.push(Token::Excl); },
            _ if c.is_alphabetic() => {
                // Analiza e inserta tokens para constantes, funciones o variables
                let name = parse_identifier(&mut chars);
                if ["pi", "e", "tau", "phi", "sqrt2", "ans", "c_light", "C_light", "h_planck", "H_planck", "g_grav", "G_grav"].contains(&name.as_str()) {
                    handle_constant(&name, &mut tokens)?;
                } else if ["sinpi", "cospi", "tanpi", "ctanpi", "sine", "cose", "tane", "ctane"].contains(&name.as_str()) {
                    handle_function_with_constant(&name, &mut tokens)?;
                } else {
                    handle_function(&name, &mut chars, &mut tokens)?;
                }
            },
            _ if c.is_whitespace() => {
                chars.next();
            },
            _ => {
                return Err(CalculatorError::UnexpectedToken);
            }
        }
    }

    Ok(preprocess_tokens(tokens))
}

/// Analiza una secuencia de caracteres y la convierte en un número de punto flotante.
/// Soporta notación científica (ej. 1.2e3, 1.2e-3).
fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<f64, CalculatorError> {
    let mut number = String::new();
    let mut dot_count = 0;
    while let Some(&c) = chars.peek() {
        if c.is_digit(10) {
            number.push(c);
            chars.next();
        } else if c == '.' {
            dot_count += 1;
            if dot_count > 1 {
                return Err(CalculatorError::ParseError);
            }
            number.push(c);
            chars.next();
        } else {
            break;
        }
    }
    // Soporte para notación científica: 1.2e3 / 1.2E-3
    if matches!(chars.peek(), Some(&'e') | Some(&'E')) {
        let mut temp_chars = chars.clone();
        temp_chars.next(); // Consume 'e'
        if matches!(temp_chars.peek(), Some(&'+') | Some(&'-')) {
            temp_chars.next(); // Consume signo
        }
        let has_digits = temp_chars.peek().map_or(false, |c| c.is_digit(10));
        
        if has_digits {
            chars.next(); // Consume 'e'
            number.push('e');
            if matches!(chars.peek(), Some(&'+') | Some(&'-')) {
                number.push(chars.next().unwrap());
            }
            while let Some(&c) = chars.peek() {
                if c.is_digit(10) {
                    number.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
        }
    }
    number.parse().map_err(|_| CalculatorError::ParseError)
}

/// Analiza expresiones logarítmicas, incluyendo aquellas con base personalizada (ej. log2(8)), e inserta los tokens respectivos.
fn parse_log(chars: &mut std::iter::Peekable<std::str::Chars>, tokens: &mut Vec<Token>) -> Result<(), CalculatorError> {
    // Comprueba directamente si sigue un dígito o '.' para determinar si se ha especificado una base.
    if chars.peek().map_or(false, |c| c.is_digit(10) || *c == '.') {
        let base = parse_number(chars)?;
 
        // Tras leer la base, comprueba si el siguiente carácter es un paréntesis de apertura.
        match chars.peek() {
            Some(&'(') => {
                // Si hay paréntesis de apertura, se utiliza el token LogBase con la base leída.
                tokens.push(Token::LogBase(base));
            },
            _ => {
                // Si no hay paréntesis de apertura, se trata como Log con paréntesis implícitos.
                tokens.push(Token::Log);
                tokens.push(Token::LeftParenthesis);
                tokens.push(Token::Number(base));
                tokens.push(Token::RightParenthesis);
            }
        }
    } else {
        // Si no hay base, se asume logaritmo común de base 10.
        tokens.push(Token::Log);
    }
    Ok(())
}

/// Extrae una secuencia consecutiva de caracteres alfabéticos de un iterador de caracteres y devuelve una cadena.
fn parse_identifier(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphabetic() || c == '_' {
            name.push(c);
            chars.next();
        } else {
            break;
        }
    }
    name
}

/// Identifica y añade tokens correspondientes a constantes matemáticas reconocidas.
fn handle_constant(name: &str, tokens: &mut Vec<Token>) -> Result<(), CalculatorError> {
    match name {
        "pi" => {
            tokens.push(Token::Pi);
            Ok(())
        },
        "e" => {
            tokens.push(Token::E);
            Ok(())
        },
        "tau" => {
            tokens.push(Token::Tau);
            Ok(())
        },
        "phi" => {
            tokens.push(Token::Phi);
            Ok(())
        },
        "sqrt2" => {
            tokens.push(Token::Sqrt2);
            Ok(())
        },
        "ans" => {
            tokens.push(Token::Ans);
            Ok(())
        },
        "c_light" | "C_light" => {
            tokens.push(Token::ConstC);
            Ok(())
        },
        "h_planck" | "H_planck" => {
            tokens.push(Token::ConstH);
            Ok(())
        },
        "g_grav" | "G_grav" => {
            tokens.push(Token::ConstG);
            Ok(())
        },
        _ => Err(CalculatorError::UnexpectedToken),
    }
}

/// Procesa nombres de funciones matemáticas reconocidas y añade los tokens correspondientes.
/// Si no coincide con ninguna función, se asume que es una variable.
fn handle_function(name: &str, chars: &mut std::iter::Peekable<std::str::Chars>, tokens: &mut Vec<Token>) -> Result<(), CalculatorError> {
    match name {
        "log" => parse_log(chars, tokens)?,
        "int" => tokens.push(Token::Int),
        "fract" => tokens.push(Token::Fract),
        "cbrt" => tokens.push(Token::Cbrt),
        "root" => tokens.push(Token::Root),
        "lcm" => tokens.push(Token::Lcm),
        "deriv" => tokens.push(Token::Deriv),
        "intg" => tokens.push(Token::Intg),
        "sum" => tokens.push(Token::Sum),
        "prod" => tokens.push(Token::Prod),
        "re" => tokens.push(Token::Re),
        "im" => tokens.push(Token::Im),
        "conj" => tokens.push(Token::Conj),
        "arg" => tokens.push(Token::Arg),
        "polar" => tokens.push(Token::Polar),
        "sin" => tokens.push(Token::Sin),
        "cos" => tokens.push(Token::Cos),
        "tan" => tokens.push(Token::Tan),
        "ctan" => tokens.push(Token::Ctan),
        "ln" => tokens.push(Token::Ln),
        "sqrt" => tokens.push(Token::Sqrt),
        "abs" => tokens.push(Token::Abs),
        "asin" | "arcsin" => tokens.push(Token::Asin),
        "acos" | "arccos" => tokens.push(Token::Acos),
        "atan" | "arctan" => tokens.push(Token::Atan),
        "sinh" => tokens.push(Token::Sinh),
        "cosh" => tokens.push(Token::Cosh),
        "tanh" => tokens.push(Token::Tanh),
        "asinh" | "arcsinh" => tokens.push(Token::Asinh),
        "acosh" | "arccosh" => tokens.push(Token::Acosh),
        "atanh" | "arctanh" => tokens.push(Token::Atanh),
        "fact" | "factorial" => tokens.push(Token::Fact),
        "floor" => tokens.push(Token::Floor),
        "ceil" => tokens.push(Token::Ceil),
        "round" => tokens.push(Token::Round),
        "trunc" => tokens.push(Token::Trunc),
        "exp" => tokens.push(Token::Exp),
        "min" => tokens.push(Token::Min),
        "max" => tokens.push(Token::Max),
        "mod" | "modulo" => tokens.push(Token::Mod),
        "gcd" => tokens.push(Token::Gcd),
        "nCr" | "ncr" | "comb" => tokens.push(Token::Ncr),
        "nPr" | "npr" | "perm" => tokens.push(Token::Npr),
        "mean" => tokens.push(Token::Mean),
        "median" => tokens.push(Token::Median),
        "var" => tokens.push(Token::VarFunc),
        "std" => tokens.push(Token::Std),
        "cov" => tokens.push(Token::Cov),
        "corr" => tokens.push(Token::Corr),
        "linreg" => tokens.push(Token::LinReg),
        "polyreg" => tokens.push(Token::PolyReg),
        "det" => tokens.push(Token::Det),
        "inv" => tokens.push(Token::Inv),
        "transpose" | "trans" => tokens.push(Token::Transpose),
        "sort" => tokens.push(Token::Sort),
        "tr" => tokens.push(Token::Tr),
        "rand" => tokens.push(Token::Rand),
        "normpdf" => tokens.push(Token::NormPdf),
        "normcdf" => tokens.push(Token::NormCdf),
        "binopdf" => tokens.push(Token::BinoPdf),
        "binocdf" => tokens.push(Token::BinoCdf),
        "poisspdf" => tokens.push(Token::PoissPdf),
        "poisscdf" => tokens.push(Token::PoissCdf),
        _ => tokens.push(Token::Variable(name.to_string())),
    }
    Ok(())
}

/// Analiza expresiones que combinan una función con una constante directamente sin operadores (ej. "sinpi" -> sin(pi)).
fn handle_function_with_constant(name: &str, tokens: &mut Vec<Token>) -> Result<(), CalculatorError> {
    let (func, const_part) = if name.ends_with("pi") {
        name.split_at(name.len() - 2)
    } else if name.ends_with('e') {
        name.split_at(name.len() - 1)
    } else {
        return Err(CalculatorError::UnexpectedToken);
    };

    match func {
        "sin" => tokens.push(Token::Sin),
        "cos" => tokens.push(Token::Cos),
        "tan" => tokens.push(Token::Tan),
        "ctan" => tokens.push(Token::Ctan),
        _ => return Err(CalculatorError::UnexpectedToken),
    }

    tokens.push(Token::LeftParenthesis);
    handle_constant(const_part, tokens)?;
    tokens.push(Token::RightParenthesis);

    Ok(())
}

/// Maneja casos donde un número es seguido inmediatamente por una constante (ej. "2pi" -> 2 * pi).
fn handle_number_constant_combination(chars: &mut std::iter::Peekable<std::str::Chars>, tokens: &mut Vec<Token>, number: f64) {
    tokens.push(Token::Number(number));

    match chars.peek() {
        Some('p') => {
            chars.next(); // Salta 'p'
            if chars.peek() == Some(&'i') {
                chars.next(); // Salta 'i'
                tokens.push(Token::Multiply);
                tokens.push(Token::Pi);
            } else {
                tokens.push(Token::Multiply);
                tokens.push(Token::Variable("p".to_string()));
            }
        },
        Some('e') => {
            chars.next(); // Salta 'e'
            tokens.push(Token::Multiply);
            tokens.push(Token::E);
        },
        Some('i') => {
            chars.next(); // Salta 'i'
            tokens.push(Token::Multiply);
            tokens.push(Token::Variable("i".to_string()));
        },
        _ => (),
    }
}

fn is_left_multiplicand(token: &Token) -> bool {
    matches!(
        token,
        Token::Number(_)
            | Token::Variable(_)
            | Token::Ans
            | Token::Pi
            | Token::E
            | Token::Tau
            | Token::Phi
            | Token::Sqrt2
            | Token::ConstC
            | Token::ConstH
            | Token::ConstG
            | Token::RightParenthesis
            | Token::RightBracket
            | Token::Excl
    )
}

fn is_right_multiplicand(token: &Token) -> bool {
    !matches!(
        token,
        Token::Plus
            | Token::Minus
            | Token::Multiply
            | Token::Divide
            | Token::Mod
            | Token::Percent
            | Token::Power
            | Token::Equal
            | Token::Excl
            | Token::RightParenthesis
            | Token::RightBracket
            | Token::Comma
            | Token::Semicolon
    )
}

fn preprocess_tokens(tokens: Vec<Token>) -> Vec<Token> {
    if crate::calculator::calculator::is_postfix_expression(&tokens) {
        return tokens;
    }

    let mut result = Vec::new();
    let mut paren_stack = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        // Trace parentheses and brackets for auto-closing
        match token {
            Token::LeftParenthesis => paren_stack.push(Token::RightParenthesis),
            Token::LeftBracket => paren_stack.push(Token::RightBracket),
            Token::RightParenthesis => {
                while let Some(top) = paren_stack.last() {
                    if *top == Token::RightParenthesis {
                        paren_stack.pop();
                        break;
                    }
                    paren_stack.pop();
                }
            }
            Token::RightBracket => {
                while let Some(top) = paren_stack.last() {
                    if *top == Token::RightBracket {
                        paren_stack.pop();
                        break;
                    }
                    paren_stack.pop();
                }
            }
            _ => {}
        }

        // Implicit multiplication insertion
        if i > 0 {
            let prev = &result[result.len() - 1];
            if is_left_multiplicand(prev) && is_right_multiplicand(token) {
                result.push(Token::Multiply);
            }
        }

        result.push(token.clone());
    }

    // Auto-close open parentheses and brackets at the end
    while let Some(closing_token) = paren_stack.pop() {
        result.push(closing_token);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_basic_arithmetic() {
        let tokens = lex("3 + 4.5").expect("Failed to lex basic arithmetic");
        assert_eq!(tokens, vec![Token::Number(3.0), Token::Plus, Token::Number(4.5)]);
    }

    #[test]
    fn test_lex_constants_and_functions() {
        let tokens = lex("sin(pi) + ln(e)").expect("Failed to lex constants and functions");
        assert_eq!(tokens, vec![Token::Sin, Token::LeftParenthesis, Token::Pi, Token::RightParenthesis, Token::Plus, Token::Ln, Token::LeftParenthesis, Token::E, Token::RightParenthesis]);
    }

    #[test]
    fn test_parse_number() {
        let mut input = "123.456".chars().peekable();
        assert_eq!(parse_number(&mut input).unwrap(), 123.456);

        let mut input = "3.14pi".chars().peekable();
        assert_eq!(parse_number(&mut input).unwrap(), 3.14); // Stops parsing at 'pi'
    }

    #[test]
    fn test_parse_log_with_explicit_base() {
        let mut chars = "100(10)".chars().peekable();
        let mut tokens = Vec::new();
        parse_log(&mut chars, &mut tokens).unwrap();
        println!("tokens{:?}", tokens);
        assert_eq!(tokens, vec![Token::LogBase(100.0)]);
    }

    #[test]
    fn test_parse_log_with_implicit_base() {
        let mut chars = "".chars().peekable();
        let mut tokens = Vec::new();
        parse_log(&mut chars, &mut tokens).unwrap();
        assert_eq!(tokens, vec![Token::Log]);
    }

    #[test]
    fn test_handle_constant_pi() {
        let mut tokens = Vec::new();
        handle_constant("pi", &mut tokens).expect("Failed to handle constant");
        assert_eq!(tokens, vec![Token::Pi]);
    }

    #[test]
    fn test_handle_function_sin() {
        let mut chars = "(".chars().peekable();
        let mut tokens = Vec::new();
        handle_function("sin", &mut chars, &mut tokens).unwrap();
        assert_eq!(tokens, vec![Token::Sin]);
    }

    #[test]
    fn test_handle_function_with_constant_sinpi() {
        let mut tokens = Vec::new();
        handle_function_with_constant("sinpi", &mut tokens).expect("Failed to handle function with constant");
        assert_eq!(tokens, vec![Token::Sin, Token::LeftParenthesis, Token::Pi, Token::RightParenthesis]);
    }

    #[test]
    fn test_handle_number_constant_combination_with_pi() {
        let mut chars = "pi+".chars().peekable();
        let mut tokens = Vec::new();
        handle_number_constant_combination(&mut chars, &mut tokens, 2.0);
        assert_eq!(tokens, vec![Token::Number(2.0), Token::Multiply, Token::Pi]);
    }

}




