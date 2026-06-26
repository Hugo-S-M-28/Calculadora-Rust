use crate::calculator::lexer::lex;
use crate::calculator::evaluator::{evaluate_infix, evaluate_postfix};
use crate::calculator::token::Token;
use crate::calculator::evaluator::solve_equation;
use crate::calculator::parser::parse;
use crate::calculator::value::Value;

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

/// Modo de ángulo global: `0` = Radianes, `1` = Grados, `2` = Gradianes.
/// Se persiste como `AtomicU8` para acceso eficiente sin bloqueo.
/// **Advertencia:** es estado global mutable; en tests usar `TEST_MUTEX` para aislar.
static ANGLE_MODE: AtomicU8 = AtomicU8::new(0);

/// Último resultado calculado, accesible mediante `ans` en expresiones.
/// Se guarda como `Mutex<f64>` para poder ser modificado de forma segura desde FFI.
static LAST_RESULT: Mutex<f64> = Mutex::new(0.0);

/// Precisión decimal global usada para redondear resultados antes de mostrarlos.
/// Valor por defecto: 8 cifras decimales. Modificable desde C# mediante `set_decimal_precision_ffi`.
static DECIMAL_PRECISION: AtomicU8 = AtomicU8::new(8);

pub(crate) fn get_angle_mode() -> u8 {
    ANGLE_MODE.load(Ordering::Relaxed)
}

/// Establece el modo de ángulo para las funciones trigonométricas.
///
/// # Parámetros
/// - `mode`: `0` = Radianes (por defecto), `1` = Grados, `2` = Gradianes.
///
/// Este valor afecta a `sin`, `cos`, `tan`, `ctan` y sus inversas `asin`, `acos`, `atan`.
/// La constante puede exponerse al lado C# mediante `set_angle_mode_ffi`.
pub fn set_angle_mode(mode: u8) {
    ANGLE_MODE.store(mode, Ordering::Relaxed);
}

/// Devuelve el valor de la variable `ans` (el último resultado calculado).
/// Retorna `0.0` si el mutex está envenenado.
pub(crate) fn get_last_result() -> f64 {
    if let Ok(guard) = LAST_RESULT.lock() {
        *guard
    } else {
        0.0
    }
}

/// Actualiza el valor de `ans` con el nuevo resultado.
/// Solo se llama cuando la expresión produce un valor escalar.
pub fn set_last_result(val: f64) {
    if let Ok(mut guard) = LAST_RESULT.lock() {
        *guard = val;
    }
}

/// Resetea `ans` a `0.0`. Útil para tests de aislamiento y para la acción "borrar todo" en la UI.
pub fn clear_last_result() {
    if let Ok(mut guard) = LAST_RESULT.lock() {
        *guard = 0.0;
    }
}

/// Enum de errores que puede retornar el motor de la calculadora.
///
/// Implementa [`std::fmt::Display`] con mensajes descriptivos en español
/// y [`std::error::Error`] para integración con el ecosistema de manejo de errores de Rust.
#[derive(Debug, PartialEq)]
pub enum CalculatorError {
    /// La expresión intenta dividir entre cero.
    DivisionByZero,
    /// La cadena de entrada no pudo ser analizada como expresión válida.
    ParseError,
    /// Se encontró un token en una posición inesperada.
    UnexpectedToken,
    /// La expresión matemática es semánticamente inválida (ej. argumento fuera de dominio).
    InvalidExpression,
    /// La ecuación contiene más de una variable desconocida y no se puede resolver.
    MultipleVariables,
    /// La expresión está vacía o solo contiene espacios en blanco.
    EmptyExpression,
    /// Hay tokens sobrantes al final de la expresión que no forman parte de ella.
    ExtraTokensDetected,
    /// Hay un paréntesis de cierre `)` sin paréntesis de apertura correspondiente.
    UnmatchedRightParenthesis,
    /// Hay un paréntesis de apertura `(` que nunca se cierra.
    UnmatchedLeftParenthesis,
}

impl std::fmt::Display for CalculatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            CalculatorError::DivisionByZero          => "División por cero",
            CalculatorError::ParseError              => "Error de análisis sintáctico",
            CalculatorError::UnexpectedToken         => "Token inesperado en la expresión",
            CalculatorError::InvalidExpression       => "Expresión no válida",
            CalculatorError::MultipleVariables       => "Se detectaron múltiples variables desconocidas",
            CalculatorError::EmptyExpression         => "La expresión está vacía",
            CalculatorError::ExtraTokensDetected     => "Tokens extra detectados al final de la expresión",
            CalculatorError::UnmatchedRightParenthesis => "Paréntesis de cierre sin par",
            CalculatorError::UnmatchedLeftParenthesis  => "Paréntesis de apertura sin par",
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for CalculatorError {}

/// Procesa una expresión matemática y devuelve el resultado como cadena de texto.
///
/// Esta es la función de entrada principal del motor. Soporta:
/// - Aritmética básica: `2 + 3 * 4`
/// - Funciones científicas: `sin(pi/2)`, `log(100)`, `sqrt(16)`
/// - Números complejos: `(3 + 4i) * (1 - 2i)`
/// - Resolución de ecuaciones lineales: `2*x + 1 = 5`
/// - Matrices y vectores: `det([1,2;3,4])`
/// - Notación posfija (RPN): `3 4 + 2 *`
/// - Funciones estadísticas: `mean([1,2,3,4])`, `normpdf(0)`
///
/// El resultado se guarda en la variable `ans` para uso en expresiones posteriores.
///
/// # Ejemplos
///
/// ```rust
/// use calculator_core::calculator::process_expression;
///
/// assert_eq!(process_expression("2 + 3").unwrap(), "5");
/// assert_eq!(process_expression("sqrt(16)").unwrap(), "4");
/// assert_eq!(process_expression("2*x + 1 = 5").unwrap(), "x=2");
/// ```
///
/// # Errors
///
/// Retorna [`CalculatorError`] en los siguientes casos:
/// - `EmptyExpression`: la entrada está vacía.
/// - `ParseError` / `UnexpectedToken`: la expresión tiene sintaxis incorrecta.
/// - `DivisionByZero`: hay una división por cero.
/// - `InvalidExpression`: argumento fuera del dominio (ej. `log(-1)`, `sqrt(-4)`).
/// - `MultipleVariables`: la ecuación tiene más de una incógnita.
pub fn process_expression(input: &str) -> Result<String, CalculatorError> {
    process_expression_ext(input, true)
}

/// Versión extendida de [`process_expression`] que permite controlar si el resultado
/// se almacena en `ans` (el acumulador del último resultado).
///
/// # Parámetros
///
/// - `input`: La expresión matemática como cadena de texto.
/// - `save_to_ans`: Si es `true`, el resultado escalar se guarda en `ans`.
///   Usar `false` para previsualizaciones en tiempo real sin modificar el estado.
///
/// # Errors
///
/// Ver [`process_expression`] para la lista completa de errores posibles.
pub fn process_expression_ext(input: &str, save_to_ans: bool) -> Result<String, CalculatorError> {
    let tokens = lex(input)?;

    if tokens.is_empty() {
        return Err(CalculatorError::EmptyExpression);
    }

    let contains_equal = tokens.iter().any(|t| *t == Token::Equal);
    let mut seen_variable = None;

    if contains_equal {
        let mut vars = Vec::new();
        for token in &tokens {
            if let Token::Variable(name) = token {
                if !vars.contains(name) {
                    vars.push(name.clone());
                }
            }
        }

        if vars.len() == 1 {
            seen_variable = Some(vars[0].clone());
        } else if vars.len() > 1 {
            let mut undefined_vars = Vec::new();
            for v in &vars {
                if crate::calculator::value::get_variable(v).is_none() {
                    undefined_vars.push(v.clone());
                }
            }

            if undefined_vars.len() == 1 {
                seen_variable = Some(undefined_vars[0].clone());
            } else if undefined_vars.is_empty() {
                if let Some(Token::Variable(lhs_name)) = tokens.first() {
                    if tokens.get(1) == Some(&Token::Equal) {
                        seen_variable = Some(lhs_name.clone());
                    }
                }
                if seen_variable.is_none() {
                    if let Some(Token::Variable(rhs_name)) = tokens.last() {
                        if tokens.get(tokens.len() - 2) == Some(&Token::Equal) {
                            seen_variable = Some(rhs_name.clone());
                        }
                    }
                }
            } else {
                return Err(CalculatorError::MultipleVariables);
            }
        }
    }

    match seen_variable {
        Some(variable_name) if contains_equal => {
            let result = solve_equation(&tokens, &variable_name)?;
            let rounded = round_result(result);
            if save_to_ans {
                set_last_result(rounded);
            }
            crate::calculator::value::set_variable(&variable_name, Value::Scalar(rounded));
            Ok(format!("{}={}", variable_name, rounded))
        },
        _ => {
            if is_postfix_expression(&tokens) {
                let result = evaluate_postfix(&tokens)?;
                if let Value::Scalar(s) = result {
                    if save_to_ans {
                        set_last_result(s);
                    }
                }
                Ok(result.to_string())
            } else {
                let (ast, _) = parse(&tokens)?;
                let result = evaluate_infix(&ast)?;
                if let Value::Scalar(s) = result {
                    if save_to_ans {
                        set_last_result(s);
                    }
                }
                Ok(result.to_string())
            }
        }
    }
}

/// Redondea un valor f64 a la precisión decimal configurada globalmente.
/// Equivalente a truncar decimales más allá de `DECIMAL_PRECISION` mediante redondeo al más próximo.
pub fn round_to_precision(val: f64) -> f64 {
    let p = DECIMAL_PRECISION.load(Ordering::Relaxed) as u32;
    let factor = 10f64.powi(p as i32);
    let rounded = (val * factor).round() / factor;
    if rounded == -0.0 { 0.0 } else { rounded }
}

/// Establece el número de decimales utilizados al redondear resultados escalares.
/// Valores válidos: 0–15. Valores mayores se redondean a 15 para evitar inestabilidad de f64.
pub fn set_decimal_precision(precision: u8) {
    DECIMAL_PRECISION.store(precision.min(15), Ordering::Relaxed);
}

fn round_result(result: f64) -> f64 {
    round_to_precision(result)
}

/// Determina si una secuencia de tokens representa una expresión en notación posfija (RPN - Notación Polaca Inversa).
pub fn is_postfix_expression(tokens: &[Token]) -> bool {
    let contains_parentheses_or_equal = tokens.iter().any(|t| matches!(t, Token::LeftParenthesis | Token::RightParenthesis | Token::Equal));
    if contains_parentheses_or_equal {
        return false;
    }

    let mut stack_depth = 0i32;
    let mut has_operator = false;

    for token in tokens {
        match token {
            Token::Number(_) | Token::Variable(_) | Token::Pi | Token::E | Token::Tau | Token::Phi | Token::Sqrt2 | Token::Ans => {
                stack_depth += 1;
            },
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Power | Token::Mod | Token::Percent => {
                if stack_depth < 2 {
                    return false;
                }
                stack_depth -= 1;
                has_operator = true;
            },
            Token::Log | Token::Ln | Token::Sin | Token::Cos | Token::Tan | Token::Ctan | Token::LogBase(_) | Token::Sqrt | Token::Abs | Token::Asin | Token::Acos | Token::Atan | Token::Sinh | Token::Cosh | Token::Tanh | Token::Asinh | Token::Acosh | Token::Atanh | Token::Fact | Token::Floor | Token::Ceil | Token::Round | Token::Trunc | Token::Exp | Token::Mean | Token::Median | Token::VarFunc | Token::Std | Token::Det | Token::Inv | Token::Transpose | Token::Int | Token::Fract | Token::Cbrt | Token::Re | Token::Im | Token::Conj | Token::Arg => {
                if stack_depth < 1 {
                    return false;
                }
                has_operator = true;
            },
            Token::Min | Token::Max | Token::Gcd | Token::Ncr | Token::Npr => {
                if stack_depth < 2 {
                    return false;
                }
                stack_depth -= 1;
                has_operator = true;
            },
            _ => return false,
        }
    }

    has_operator && stack_depth == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculator::TEST_MUTEX;

    #[test]
    fn test_basic_operations() {
        let _guard = TEST_MUTEX.lock().unwrap();
        assert_eq!(process_expression("1 + 1"), Ok("2".to_string()));
        assert_eq!(process_expression("2 - 1"), Ok("1".to_string()));
        assert_eq!(process_expression("2 * 3"), Ok("6".to_string()));
        assert_eq!(process_expression("8 / 4"), Ok("2".to_string()));
    }

    #[test]
    fn test_complex_expressions() {
        let _guard = TEST_MUTEX.lock().unwrap();
        assert_eq!(process_expression("2 * (3 + 4)"), Ok("14".to_string()));
        assert_eq!(process_expression("(2 + 3) * (4 - 1)"), Ok("15".to_string()));
    }

    #[test]
    fn test_trigonometric_functions() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        assert_eq!(process_expression("cos(0)"), Ok("1".to_string()));
        assert_eq!(process_expression("tan(pi/4)"), Ok("1".to_string()));
    }

    #[test]
    fn test_logarithmic_functions() {
        let _guard = TEST_MUTEX.lock().unwrap();
        assert_eq!(process_expression("ln(e)"), Ok("1".to_string()));
        assert_eq!(process_expression("log(100)"), Ok("2".to_string()));
    }

    #[test]
    fn test_error_handling() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        assert!(process_expression("2 / 0").is_err());
        assert!(process_expression("2 * (3 + 4))").is_err()); // Unmatched right parenthesis
        assert!(process_expression("sin(90))").is_err()); // Unmatched right parenthesis
    }

    #[test]
    fn test_implicit_multiplication_and_autoclose() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        // Autoclose test
        assert_eq!(process_expression("2 * (3 + 4"), Ok("14".to_string()));
        assert_eq!(process_expression("sin(pi/2"), Ok("1".to_string()));
        assert_eq!(process_expression("sqrt(9"), Ok("3".to_string()));

        // Implicit multiplication test
        assert_eq!(process_expression("5sqrt(9)"), Ok("15".to_string()));
        assert_eq!(process_expression("5(3+2)"), Ok("25".to_string()));
        assert_eq!(process_expression("(2+3)(4-1)"), Ok("15".to_string()));
        assert_eq!(process_expression("2pi"), Ok("6.28318531".to_string()));
        assert_eq!(process_expression("pi 2"), Ok("6.28318531".to_string()));
    }

    #[test]
    fn test_constants_and_variables() {
        let _guard = TEST_MUTEX.lock().unwrap();
        assert_eq!(process_expression("pi"), Ok("3.14159265".to_string()));
        assert_eq!(process_expression("e"), Ok("2.71828183".to_string()));
        assert_eq!(process_expression("2 * x + 1 = 3"), Ok("x=1".to_string()));
    }

    #[test]
    fn evaluate_simple_expression() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let input = "(3+(4-1))*5";
        let result = process_expression(input);
        assert_eq!(result, Ok("30".to_string()));
    }

    #[test]
    fn solve_linear_equation() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let input = "2 * x + 0.5 = 1";
        let result = process_expression(input);
        assert_eq!(result, Ok("x=0.25".to_string()));
    }

    #[test]
    fn solve_equation_with_variables_on_both_sides() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let input = "2 * x + 1 = 2 * (1 - x)";
        let result = process_expression(input);
        assert_eq!(result, Ok("x=0.25".to_string()));
    }

    #[test]
    fn test_log_base_10_of_10() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let input = "log(10)";
        assert_eq!(process_expression(input), Ok("1".to_string()));

        let input = "log10";
        assert_eq!(process_expression(input), Ok("1".to_string()));
    }

    #[test]
    fn test_log_base_100_of_10() {
        let _guard = TEST_MUTEX.lock().unwrap();
        let input = "log100(10)";
        assert_eq!(process_expression(input), Ok("0.5".to_string()));
    }

    #[test]
    fn test_sin_of_pi() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        let input = "sin(pi)";
        assert_eq!(process_expression(input), Ok("0".to_string()));

        let input = "sinpi";
        assert_eq!(process_expression(input), Ok("0".to_string()));
    }

    #[test]
    fn test_sin_of_1_5_pi() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        let input = "sin(1.5pi)";
        assert_eq!(process_expression(input), Ok("-1".to_string()));

        let input = "sin(1.5*pi)";
        assert_eq!(process_expression(input), Ok("-1".to_string()));
    }

    #[test]
    fn test_postfix_expression() {
        let _guard = TEST_MUTEX.lock().unwrap();
        assert_eq!(process_expression("3 4 + 2 *"), Ok("14".to_string()));
        assert_eq!(process_expression("10 2 8 * + 3 -"), Ok("23".to_string()));
        assert_eq!(process_expression("2 3 4 * +"), Ok("14".to_string()));
    }

    #[test]
    fn test_power_operation() {
        let _guard = TEST_MUTEX.lock().unwrap();
        assert_eq!(process_expression("2 ^ 3"), Ok("8".to_string()));
        assert_eq!(process_expression("2 ^ 3 ^ 2"), Ok("512".to_string())); // 2^(3^2) = 2^9 = 512
        assert_eq!(process_expression("2 3 ^"), Ok("8".to_string()));
    }

    #[test]
    fn test_sqrt_and_abs_functions() {
        let _guard = TEST_MUTEX.lock().unwrap();
        assert_eq!(process_expression("sqrt(16)"), Ok("4".to_string()));
        assert_eq!(process_expression("abs(-5.5)"), Ok("5.5".to_string()));
        assert_eq!(process_expression("9 sqrt"), Ok("3".to_string()));
        assert_eq!(process_expression("0 9 - abs sqrt"), Ok("3".to_string()));
    }

    #[test]
    fn test_new_scientific_functions() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        assert_eq!(process_expression("asin(1)"), Ok("1.57079633".to_string()));
        assert_eq!(process_expression("acos(1)"), Ok("0".to_string()));
        assert_eq!(process_expression("atan(1)"), Ok("0.78539816".to_string()));
        assert_eq!(process_expression("sinh(0)"), Ok("0".to_string()));
        assert_eq!(process_expression("cosh(0)"), Ok("1".to_string()));
        assert_eq!(process_expression("tanh(0)"), Ok("0".to_string()));
        assert_eq!(process_expression("fact(5)"), Ok("120".to_string()));
        assert_eq!(process_expression("fact(0)"), Ok("1".to_string()));
        assert!(process_expression("fact(-1)").is_err());
        assert!(process_expression("fact(1.5)").is_err());
        assert_eq!(process_expression("5 fact"), Ok("120".to_string()));
        assert_eq!(process_expression("0 sinh"), Ok("0".to_string()));
    }

    #[test]
    fn test_phase1_and_phase2_improvements() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        // Redondeo int y fract
        assert_eq!(process_expression("int(5.7)"), Ok("5".to_string()));
        assert_eq!(process_expression("int(-3.2)"), Ok("-3".to_string()));
        assert_eq!(process_expression("fract(5.75)"), Ok("0.75".to_string()));
        assert_eq!(process_expression("fract(-3.2)"), Ok("-0.2".to_string()));

        // Raíces
        assert_eq!(process_expression("cbrt(8)"), Ok("2".to_string()));
        assert_eq!(process_expression("cbrt(-27)"), Ok("-3".to_string()));
        assert_eq!(process_expression("root(16, 4)"), Ok("2".to_string()));
        assert_eq!(process_expression("root(-8, 3)"), Ok("-2".to_string()));

        // Aritmética mcm (lcm)
        assert_eq!(process_expression("lcm(12, 18)"), Ok("36".to_string()));
        assert_eq!(process_expression("lcm(5, 7)"), Ok("35".to_string()));

        // Constantes C_light, c_light, G_grav (antiguas C, G)
        assert_eq!(process_expression("C_light"), Ok("299792458".to_string()));
        assert_eq!(process_expression("c_light"), Ok("299792458".to_string()));
        assert!(process_expression("G_grav").is_ok());

        // Factorial posfijo !
        assert_eq!(process_expression("5!"), Ok("120".to_string()));
        assert_eq!(process_expression("0!"), Ok("1".to_string()));
        assert_eq!(process_expression("3! + 4!"), Ok("30".to_string()));

        // Variables A..Z y θ
        assert_eq!(process_expression("10 = A"), Ok("A=10".to_string()));
        assert_eq!(process_expression("A * 3"), Ok("30".to_string()));
        assert_eq!(process_expression("A = B"), Ok("B=10".to_string()));
        assert_eq!(process_expression("B - 5"), Ok("5".to_string()));

        // Ángulo Gradianes (GRAD = 2)
        set_angle_mode(2);
        assert_eq!(process_expression("sin(100)"), Ok("1".to_string()));
        assert_eq!(process_expression("asin(1)"), Ok("100".to_string()));
        set_angle_mode(0);
    }

    #[test]
    fn test_complex_numbers() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        // Basic complex number parsing and operations
        assert_eq!(process_expression("3 + 4i"), Ok("3+4i".to_string()));
        assert_eq!(process_expression("(3 + 4i) + (1 - 2i)"), Ok("4+2i".to_string()));
        assert_eq!(process_expression("(1 + i) * (1 - i)"), Ok("2".to_string()));
        assert_eq!(process_expression("re(3 + 4i)"), Ok("3".to_string()));
        assert_eq!(process_expression("im(3 + 4i)"), Ok("4".to_string()));
        assert_eq!(process_expression("conj(3 + 4i)"), Ok("3-4i".to_string()));
        assert_eq!(process_expression("abs(3 + 4i)"), Ok("5".to_string()));
        // Polar complex number
        assert_eq!(process_expression("polar(5, 0)"), Ok("5".to_string()));
    }

    #[test]
    fn test_numerical_analysis() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        // Numerical derivative of x^2 at x = 3 is 6
        assert_eq!(process_expression("deriv(x^2, x, 3)"), Ok("6".to_string()));
        // Implicit x variable
        assert_eq!(process_expression("deriv(x^2, 3)"), Ok("6".to_string()));

        // Numerical integration of x^2 from 0 to 3 is 9
        assert_eq!(process_expression("intg(x^2, x, 0, 3)"), Ok("9".to_string()));
        // Implicit x variable
        assert_eq!(process_expression("intg(x^2, 0, 3)"), Ok("9".to_string()));

        // Summation: sum(i^2, i, 1, 4) = 1 + 4 + 9 + 16 = 30
        assert_eq!(process_expression("sum(i^2, i, 1, 4)"), Ok("30".to_string()));

        // Product: prod(i, i, 1, 5) = 120
        assert_eq!(process_expression("prod(i, i, 1, 5)"), Ok("120".to_string()));
    }

    #[test]
    fn test_statistics_and_matrices() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        // Vectors and Matrices parsing
        assert_eq!(process_expression("[1, 2, 3]"), Ok("[1, 2, 3]".to_string()));
        assert_eq!(process_expression("[1, 2; 3, 4]"), Ok("[1, 2; 3, 4]".to_string()));
        assert_eq!(process_expression("[1+2, 3*4]"), Ok("[3, 12]".to_string()));

        // Statistics
        assert_eq!(process_expression("mean([1, 2, 3, 4])"), Ok("2.5".to_string()));
        assert_eq!(process_expression("median([5, 1, 3])"), Ok("3".to_string()));
        assert_eq!(process_expression("var([1, 2, 3])"), Ok("1".to_string()));
        assert_eq!(process_expression("std([1, 2, 3])"), Ok("1".to_string()));

        // Binary statistics and regressions
        assert_eq!(process_expression("cov([1, 2, 3], [2, 4, 6])"), Ok("2".to_string()));
        assert_eq!(process_expression("corr([1, 2, 3], [2, 4, 6])"), Ok("1".to_string()));
        assert_eq!(process_expression("linreg([1, 2, 3], [2, 4, 6])"), Ok("[2, 0, 1, 1]".to_string()));

        // Polynomial regression: y = x^2
        assert_eq!(process_expression("polyreg([1, 2, 3], [1, 4, 9], 2)"), Ok("[1, 0, 0, 1]".to_string()));

        // Matrix linear algebra operations
        assert_eq!(process_expression("det([1, 2; 3, 4])"), Ok("-2".to_string()));
        assert_eq!(process_expression("inv([1, 2; 3, 4])"), Ok("[-2, 1; 1.5, -0.5]".to_string()));
        assert_eq!(process_expression("transpose([1, 2; 3, 4])"), Ok("[1, 3; 2, 4]".to_string()));
        assert_eq!(process_expression("transpose([1, 2, 3])"), Ok("[1, 2, 3]".to_string()));
        assert_eq!(process_expression("det(5)"), Ok("5".to_string()));
        assert_eq!(process_expression("inv(2)"), Ok("0.5".to_string()));
        assert_eq!(process_expression("transpose(5)"), Ok("5".to_string()));
    }

    #[test]
    fn test_probability_functions() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);
        
        // Rand
        let rand_val_str = process_expression("rand()").unwrap();
        let rand_val: f64 = rand_val_str.parse().unwrap();
        assert!(rand_val >= 0.0 && rand_val <= 1.0);
        
        let rand_val_str2 = process_expression("rand(10)").unwrap();
        let rand_val2: f64 = rand_val_str2.parse().unwrap();
        assert!(rand_val2 >= 0.0 && rand_val2 <= 10.0);

        let rand_val_str3 = process_expression("rand(5, 10)").unwrap();
        let rand_val3: f64 = rand_val_str3.parse().unwrap();
        assert!(rand_val3 >= 5.0 && rand_val3 <= 10.0);

        // NormPdf and NormCdf
        // Standard normal pdf at 0 is 1/sqrt(2*pi) ~= 0.39894228
        assert_eq!(process_expression("normpdf(0)"), Ok("0.39894228".to_string()));
        // Standard normal cdf at 0 is 0.5
        assert_eq!(process_expression("normcdf(0)"), Ok("0.5".to_string()));
        
        // General normal pdf(0, 0, 2) = pdf of N(0, 2) at 0 is 1/(2*sqrt(2*pi)) ~= 0.19947114
        assert_eq!(process_expression("normpdf(0, 0, 2)"), Ok("0.19947114".to_string()));
        assert_eq!(process_expression("normcdf(0, 0, 2)"), Ok("0.5".to_string()));

        // BinoPdf and BinoCdf
        // Binomial with n=10, p=0.5. PMF at k=5 is 10C5 * 0.5^10 = 252 * 0.5^10 ~= 0.24609375
        assert_eq!(process_expression("binopdf(5, 10, 0.5)"), Ok("0.24609375".to_string()));
        // Cumulative cdf(5, 10, 0.5) is 0.62304687
        assert_eq!(process_expression("binocdf(5, 10, 0.5)"), Ok("0.62304687".to_string()));

        // PoissPdf and PoissCdf
        // Poisson with lambda=2. PMF at k=1 is 2^1 * e^-2 / 1! = 2 * e^-2 ~= 0.27067057
        assert_eq!(process_expression("poisspdf(1, 2)"), Ok("0.27067057".to_string()));
        // Cumulative cdf(1, 2) is e^-2 + 2*e^-2 = 3 * e^-2 ~= 0.40600585
        assert_eq!(process_expression("poisscdf(1, 2)"), Ok("0.40600585".to_string()));
    }

    #[test]
    fn test_fase_1_fixes() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);

        // BUG-1: c, g, h son variables y no colisionan con constantes
        assert_eq!(process_expression("c = 5"), Ok("c=5".to_string()));
        assert_eq!(process_expression("c * 2"), Ok("10".to_string()));
        assert_eq!(process_expression("h = 10"), Ok("h=10".to_string()));
        assert_eq!(process_expression("h - 2"), Ok("8".to_string()));
        assert_eq!(process_expression("g = 3"), Ok("g=3".to_string()));
        assert_eq!(process_expression("g + 1"), Ok("4".to_string()));

        // BUG-9: Precedencia de - unario vs potencia
        assert_eq!(process_expression("-2^2"), Ok("-4".to_string()));
        assert_eq!(process_expression("-(2)^2"), Ok("-4".to_string()));
        assert_eq!(process_expression("(-2)^2"), Ok("4".to_string()));
        assert_eq!(process_expression("2^-2"), Ok("0.25".to_string()));
        assert_eq!(process_expression("-2^-2"), Ok("-0.25".to_string()));

        // BUG-10: Funciones trigonométricas con constante 'e'
        // sine -> sin(e) -> sin(2.718281828459045) ~= 0.41078129
        assert_eq!(process_expression("sine"), Ok("0.41078129".to_string()));
        // cose -> cos(e) -> cos(2.718281828459045) ~= -0.91173391
        assert_eq!(process_expression("cose"), Ok("-0.91173391".to_string()));
    }

    #[test]
    fn test_fase_2_fixes() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);

        // BUG-2: p no consumido silenciosamente se trata como variable
        assert_eq!(process_expression("p = 5"), Ok("p=5".to_string()));
        assert_eq!(process_expression("3p"), Ok("15".to_string()));

        // BUG-3: Notación científica vs constante Euler e
        assert_eq!(process_expression("3e"), Ok("8.15484549".to_string())); // 3 * e ~= 8.15484549
        assert_eq!(process_expression("3e2"), Ok("300".to_string())); // notación científica

        // BUG-4 y BUG-5: No enmascarar errores de parseo y único punto decimal
        assert!(process_expression("1.2.3").is_err());

        // BUG-11: gcd / lcm desbordamientos y valores no finitos
        assert!(process_expression("gcd(1e30, 2)").is_err());
        assert!(process_expression("gcd(0/0, 2)").is_err());
        assert!(process_expression("lcm(1e30, 2)").is_err());
    }

    #[test]
    fn test_fase_3_fixes() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);

        // MEJ-1: Display legible de errores (no variantes Rust crudas como "ParseError")
        let err_msg = format!("{}", process_expression("2 +").unwrap_err());
        assert!(!err_msg.contains("ParseError"), "El mensaje de error debe ser legible, no un Debug de variante Rust");
        assert!(!err_msg.is_empty(), "El mensaje de error no debe estar vacío");

        // MEJ-4: Validación de dominio — log de negativo/cero debe dar error
        assert!(process_expression("log(-1)").is_err(),  "log(-1) debe ser error de dominio");
        assert!(process_expression("log(0)").is_err(),   "log(0) debe ser error de dominio");
        assert!(process_expression("ln(-1)").is_err(),   "ln(-1) debe ser error de dominio");
        assert!(process_expression("ln(0)").is_err(),    "ln(0) debe ser error de dominio");

        // MEJ-4: sqrt de negativo ahora devuelve un número complejo
        assert_eq!(process_expression("sqrt(-4)"), Ok("2i".to_string()));

        // MEJ-4: LogBase con dominio inválido
        assert!(process_expression("log100(-5)").is_err(), "log_base de negativo debe ser error");

        // MEJ-4: Valores positivos siguen funcionando correctamente
        assert_eq!(process_expression("log(1000)"), Ok("3".to_string()));
        assert_eq!(process_expression("ln(1)"),     Ok("0".to_string()));
        assert_eq!(process_expression("sqrt(25)"),  Ok("5".to_string()));
    }

    #[test]
    fn test_fase_5_edge_cases() {
        let _guard = TEST_MUTEX.lock().unwrap();
        set_angle_mode(0);

        // MEJ-12: factorial de 171+ ahora devuelve Infinity en lugar de error
        assert_eq!(process_expression("171!"), Ok("Infinity".to_string()));

        // MEJ-12: expresión vacía con paréntesis
        assert!(process_expression("()").is_err(), "() no es una expresión válida");

        // MEJ-12: matrices no cuadradas con det() deben dar error
        assert!(process_expression("det([1,2,3;4,5,6])").is_err(),
            "det de matriz no cuadrada debe dar error");

        // MEJ-12: overflow de f64 (Inf) — no debe causar pánico
        let big = process_expression("1e308 * 10");
        assert!(big.is_ok() || big.is_err(), "expresiones con Inf no deben causar pánico");

        // MEJ-12: división por cero en diferentes contextos
        assert!(process_expression("1/0").is_err(),   "1/0 debe ser error");
        assert!(process_expression("0/0").is_err(),   "0/0 debe ser error");
        assert!(process_expression("-1/0").is_err(),  "-1/0 debe ser error");

        // MEJ-12: operaciones con resultado correcto en casos borde normales
        assert_eq!(process_expression("0^0"), Ok("1".to_string()));      // convenio matemático
        assert_eq!(process_expression("1/1"), Ok("1".to_string()));
        assert_eq!(process_expression("-0"), Ok("0".to_string()));

        // MEJ-12: factorial de valores válidos grandes
        let fact_10 = process_expression("10!").unwrap();
        assert_eq!(fact_10, "3628800");

        // MEJ-16: clear_variables debe restaurar i y j
        use crate::calculator::value::{clear_variables, get_variable};
        clear_variables();
        let i_val = get_variable("i");
        let j_val = get_variable("j");
        assert!(i_val.is_some(), "clear_variables debe restaurar la variable i");
        assert!(j_val.is_some(), "clear_variables debe restaurar la variable j");

        // Después de clear, i sigue funcionando en expresiones
        assert!(process_expression("3*i").is_ok(), "i debe estar disponible tras clear_variables");
    }

    // ======================== FASE 4 — Rendimiento y usabilidad ========================
    #[test]
    fn phase4_mej10_decimal_precision() {
        // Precision default = 8 decimales
        set_decimal_precision(8);
        let v = round_to_precision(1.0 / 3.0);
        assert_eq!(v, 0.33333333);

        // Cambiar a 4 decimales
        set_decimal_precision(4);
        let v2 = round_to_precision(1.0 / 3.0);
        assert_eq!(v2, 0.3333);

        // Restaurar
        set_decimal_precision(8);
    }

    #[test]
    fn phase4_mej11_power_alias() {
        // ** debe comportarse igual que ^
        let r1 = process_expression("2**10").unwrap();
        let r2 = process_expression("2^10").unwrap();
        assert_eq!(r1, r2, "2**10 debe ser igual que 2^10");
        assert_eq!(r1, "1024");
    }

    #[test]
    fn phase4_mej5_ncr_large() {
        // C(100,50) no debe hacer overflow de f64 con la versión iterativa
        let r = process_expression("nCr(100,50)");
        assert!(r.is_ok(), "nCr(100,50) no debe fallar: {:?}", r);
        // El resultado real es ~1.009e29, verificamos que sea un número
        let v: f64 = r.unwrap().parse().unwrap();
        assert!(v.is_finite() && v > 0.0, "nCr(100,50) debe ser un número positivo finito");
    }

    #[test]
    fn phase4_mej5_npr_large() {
        // P(20,10) no debe overflow
        let r = process_expression("nPr(20,10)");
        assert!(r.is_ok(), "nPr(20,10) no debe fallar: {:?}", r);
        let v: f64 = r.unwrap().parse().unwrap();
        assert!(v.is_finite() && v > 0.0, "nPr(20,10) debe ser positivo y finito");
    }
}
