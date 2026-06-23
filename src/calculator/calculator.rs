use crate::calculator::lexer::lex;
use crate::calculator::evaluator::{evaluate_infix, evaluate_postfix};
use crate::calculator::token::Token;
use crate::calculator::evaluator::solve_equation;
use crate::calculator::parser::parse;
use crate::calculator::value::Value;

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

static ANGLE_MODE: AtomicU8 = AtomicU8::new(0); // 0 = Radian, 1 = Degree, 2 = Gradian
static LAST_RESULT: Mutex<f64> = Mutex::new(0.0);

pub(crate) fn get_angle_mode() -> u8 {
    ANGLE_MODE.load(Ordering::Relaxed)
}

pub fn set_angle_mode(mode: u8) {
    ANGLE_MODE.store(mode, Ordering::Relaxed);
}

pub(crate) fn get_last_result() -> f64 {
    if let Ok(guard) = LAST_RESULT.lock() {
        *guard
    } else {
        0.0
    }
}

pub fn set_last_result(val: f64) {
    if let Ok(mut guard) = LAST_RESULT.lock() {
        *guard = val;
    }
}

pub fn clear_last_result() {
    if let Ok(mut guard) = LAST_RESULT.lock() {
        *guard = 0.0;
    }
}

#[derive(Debug, PartialEq)]
pub enum CalculatorError {
    DivisionByZero,
    ParseError,
    UnexpectedToken,
    InvalidExpression,
    MultipleVariables,
    EmptyExpression,
    ExtraTokensDetected,
    UnmatchedRightParenthesis,
    UnmatchedLeftParenthesis,
}

/// Procesa una expresión matemática dada en formato de cadena de texto y devuelve el resultado como una cadena.
pub fn process_expression(input: &str) -> Result<String, CalculatorError> {
    process_expression_ext(input, true)
}

/// Versión extendida de process_expression que permite evitar guardar el resultado en 'ans'.
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

fn round_result(result: f64) -> f64 {
    (result * 100000000.0).round() / 100000000.0
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
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

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

        // Constantes C, H, G
        assert_eq!(process_expression("C"), Ok("299792458".to_string()));
        assert_eq!(process_expression("c"), Ok("299792458".to_string()));
        assert!(process_expression("G").is_ok());

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
        assert_eq!(process_expression("3 + 4i"), Ok("3 + 4i".to_string()));
        assert_eq!(process_expression("(3 + 4i) + (1 - 2i)"), Ok("4 + 2i".to_string()));
        assert_eq!(process_expression("(1 + i) * (1 - i)"), Ok("2".to_string()));
        assert_eq!(process_expression("re(3 + 4i)"), Ok("3".to_string()));
        assert_eq!(process_expression("im(3 + 4i)"), Ok("4".to_string()));
        assert_eq!(process_expression("conj(3 + 4i)"), Ok("3 - 4i".to_string()));
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
}
