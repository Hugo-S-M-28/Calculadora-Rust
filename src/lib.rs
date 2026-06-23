pub mod calculator;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::calculator::lexer::lex;
use crate::calculator::parser::parse;
use crate::calculator::evaluator::{evaluate_postfix, evaluate_infix};
use crate::calculator::calculator::is_postfix_expression;
use crate::calculator::token::Token;
use crate::calculator::ast::AST;


/// Estructura de resultado para FFI compatible con C# que separa el resultado y el estado de éxito.
#[repr(C)]
pub struct FfiCalculatorResult {
    pub value: *mut c_char,
    pub is_success: u8,
}

/// Punto de entrada compatible con C para procesar expresiones matemáticas.
/// Utiliza `catch_unwind` para asegurar que ningún pánico de Rust cruce la frontera FFI (lo que causaría comportamiento indefinido en .NET).
#[no_mangle]
pub extern "C" fn process_expression_ffi(input: *const c_char) -> FfiCalculatorResult {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if input.is_null() {
            return FfiCalculatorResult {
                value: CString::new("Error: Puntero de entrada nulo").unwrap().into_raw(),
                is_success: 0,
            };
        }
        
        let c_str = unsafe { CStr::from_ptr(input) };
        let input_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return FfiCalculatorResult {
                value: CString::new("Error: Cadena UTF-8 no válida").unwrap().into_raw(),
                is_success: 0,
            },
        };

        match calculator::process_expression(input_str) {
            Ok(res) => FfiCalculatorResult {
                value: CString::new(res).unwrap().into_raw(),
                is_success: 1,
            },
            Err(e) => FfiCalculatorResult {
                value: CString::new(format!("Error: {:?}", e)).unwrap().into_raw(),
                is_success: 0,
            },
        }
    }));

    match panic_res {
        Ok(res) => res,
        Err(_) => FfiCalculatorResult {
            value: CString::new("Error: Pánico interno en el motor Rust").unwrap().into_raw(),
            is_success: 0,
        },
    }
}

/// Punto de entrada compatible con C para procesar previsualizaciones de expresiones
/// sin alterar el estado del último resultado calculado (ans).
#[no_mangle]
pub extern "C" fn process_expression_preview_ffi(input: *const c_char) -> FfiCalculatorResult {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if input.is_null() {
            return FfiCalculatorResult {
                value: CString::new("Error: Puntero de entrada nulo").unwrap().into_raw(),
                is_success: 0,
            };
        }
        
        let c_str = unsafe { CStr::from_ptr(input) };
        let input_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return FfiCalculatorResult {
                value: CString::new("Error: Cadena UTF-8 no válida").unwrap().into_raw(),
                is_success: 0,
            },
        };

        match calculator::process_expression_ext(input_str, false) {
            Ok(res) => FfiCalculatorResult {
                value: CString::new(res).unwrap().into_raw(),
                is_success: 1,
            },
            Err(e) => FfiCalculatorResult {
                value: CString::new(format!("Error: {:?}", e)).unwrap().into_raw(),
                is_success: 0,
            },
        }
    }));

    match panic_res {
        Ok(res) => res,
        Err(_) => FfiCalculatorResult {
            value: CString::new("Error: Pánico interno en el motor Rust").unwrap().into_raw(),
            is_success: 0,
        },
    }
}

/// Valida si una expresión matemática tiene una sintaxis correcta.
/// Retorna 1 si es válida, 0 en caso contrario.
#[no_mangle]
pub extern "C" fn validate_expression_ffi(input: *const c_char) -> u8 {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if input.is_null() {
            return 0;
        }
        
        let c_str = unsafe { CStr::from_ptr(input) };
        let input_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        if input_str.trim().is_empty() {
            return 0;
        }

        // Intenta realizar el análisis léxico (Lexing)
        let tokens = match lex(input_str) {
            Ok(t) => t,
            Err(_) => return 0,
        };

        if tokens.is_empty() {
            return 0;
        }

        // Verifica si contiene una igualdad '='
        let contains_equal = tokens.iter().any(|t| *t == Token::Equal);
        if contains_equal {
            let equal_count = tokens.iter().filter(|t| **t == Token::Equal).count();
            if equal_count != 1 {
                return 0; // Las ecuaciones deben tener exactamente un signo '='
            }

            // Comprueba que no haya múltiples variables diferentes en la ecuación
            let mut seen_variable = None;
            for token in &tokens {
                if let Token::Variable(name) = token {
                    match seen_variable {
                        None => seen_variable = Some(name.clone()),
                        Some(ref seen_name) if seen_name != name => {
                            return 0; // No se permiten múltiples variables en la ecuación
                        },
                        _ => {}
                    }
                }
            }

            let pos = tokens.iter().position(|t| *t == Token::Equal).unwrap();
            let left = &tokens[..pos];
            let right = &tokens[pos+1..];
            if left.is_empty() || right.is_empty() {
                return 0;
            }
            if parse(left).is_err() || parse(right).is_err() {
                return 0;
            }
            return 1;
        }

        // Valida según el tipo de notación
        if is_postfix_expression(&tokens) {
            // Validador de notación posfija (RPN)
            match evaluate_postfix(&tokens) {
                Ok(_) | Err(crate::calculator::calculator::CalculatorError::DivisionByZero) => 1,
                _ => 0,
            }
        } else {
            // Validador de notación infija estándar
            match parse(&tokens) {
                Ok((_, rest)) => if rest.is_empty() { 1 } else { 0 },
                Err(_) => 0,
            }
        }
    }));

    match panic_res {
        Ok(val) => val,
        Err(_) => 0,
    }
}

/// Convertidor de unidades físico-químicas a través de FFI.
/// Recibe el valor numérico, las unidades de origen y destino, y la categoría.
#[no_mangle]
pub extern "C" fn convert_units_ffi(
    value: f64,
    from_unit: *const c_char,
    to_unit: *const c_char,
    category: *const c_char
) -> f64 {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if from_unit.is_null() || to_unit.is_null() || category.is_null() {
            return 0.0;
        }

        let from_str = match unsafe { CStr::from_ptr(from_unit) }.to_str() {
            Ok(s) => s,
            Err(_) => return 0.0,
        };

        let to_str = match unsafe { CStr::from_ptr(to_unit) }.to_str() {
            Ok(s) => s,
            Err(_) => return 0.0,
        };

        let cat_str = match unsafe { CStr::from_ptr(category) }.to_str() {
            Ok(s) => s,
            Err(_) => return 0.0,
        };

        match cat_str {
            "length" => {
                let from_factor = match get_length_factor(from_str) {
                    Some(f) => f,
                    None => return 0.0,
                };
                let to_factor = match get_length_factor(to_str) {
                    Some(f) => f,
                    None => return 0.0,
                };
                (value * from_factor) / to_factor
            },
            "temperature" => {
                match convert_temp(value, from_str, to_str) {
                    Some(t) => t,
                    None => 0.0,
                }
            },
            "mass" => {
                let from_factor = match get_mass_factor(from_str) {
                    Some(f) => f,
                    None => return 0.0,
                };
                let to_factor = match get_mass_factor(to_str) {
                    Some(f) => f,
                    None => return 0.0,
                };
                (value * from_factor) / to_factor
            },
            "volume" => {
                let from_factor = match get_volume_factor(from_str) {
                    Some(f) => f,
                    None => return 0.0,
                };
                let to_factor = match get_volume_factor(to_str) {
                    Some(f) => f,
                    None => return 0.0,
                };
                (value * from_factor) / to_factor
            },
            _ => 0.0,
        }
    }));

    match panic_res {
        Ok(val) => val,
        Err(_) => 0.0,
    }
}

/// Función auxiliar para obtener factores de conversión de longitud con respecto al Metro (m).
fn get_length_factor(unit: &str) -> Option<f64> {
    match unit {
        "m" => Some(1.0),
        "km" => Some(1000.0),
        "cm" => Some(0.01),
        "mm" => Some(0.001),
        "in" => Some(0.0254),
        "ft" => Some(0.3048),
        "yd" => Some(0.9144),
        "mi" => Some(1609.344),
        _ => None,
    }
}

/// Función auxiliar para obtener factores de conversión de masa con respecto al Gramo (g).
fn get_mass_factor(unit: &str) -> Option<f64> {
    match unit {
        "g" => Some(1.0),
        "kg" => Some(1000.0),
        "mg" => Some(0.001),
        "t" => Some(1000000.0),
        "lb" => Some(453.59237),
        "oz" => Some(28.349523125),
        _ => None,
    }
}

/// Función auxiliar para obtener factores de conversión de volumen con respecto al Litro (L).
fn get_volume_factor(unit: &str) -> Option<f64> {
    match unit {
        "L" => Some(1.0),
        "mL" => Some(0.001),
        "gal" => Some(3.785411784),
        "qt" => Some(0.946352946),
        "pt" => Some(0.473176473),
        "cup" => Some(0.236588236),
        "m3" | "m³" => Some(1000.0),
        "ft3" | "ft³" => Some(28.316846592),
        _ => None,
    }
}

/// Función auxiliar para realizar conversiones de temperatura entre escalas (°C, °F, K).
fn convert_temp(val: f64, from: &str, to: &str) -> Option<f64> {
    // 1. Convertir a escala base Celsius
    let celsius = match from {
        "C" => val,
        "F" => (val - 32.0) / 1.8,
        "K" => val - 273.15,
        _ => return None,
    };
    // 2. Convertir desde Celsius al destino solicitado
    let target = match to {
        "C" => celsius,
        "F" => celsius * 1.8 + 32.0,
        "K" => celsius + 273.15,
        _ => return None,
    };
    Some(target)
}

/// Recorre el AST y reemplaza la variable x por el valor numérico especificado.
fn substitute_variable(ast: &AST, var_name: &str, val: f64) -> AST {
    match ast {
        AST::Num(n) => AST::Num(*n),
        AST::Var(name) => {
            if name == var_name {
                AST::Num(val)
            } else {
                AST::Var(name.clone())
            }
        },
        AST::BinOp(lhs, op, rhs) => {
            AST::BinOp(
                Box::new(substitute_variable(lhs, var_name, val)),
                op.clone(),
                Box::new(substitute_variable(rhs, var_name, val))
            )
        },
        AST::Func(func, arg) => {
            AST::Func(
                func.clone(),
                Box::new(substitute_variable(arg, var_name, val))
            )
        },
        AST::Const(c) => AST::Const(*c),
        AST::LogBase(base, expr) => {
            AST::LogBase(
                *base,
                Box::new(substitute_variable(expr, var_name, val))
            )
        },
        AST::Func2(func, lhs, rhs) => {
            AST::Func2(
                func.clone(),
                Box::new(substitute_variable(lhs, var_name, val)),
                Box::new(substitute_variable(rhs, var_name, val))
            )
        },
        AST::Deriv(expr, var, point) => {
            AST::Deriv(
                if var == var_name { expr.clone() } else { Box::new(substitute_variable(expr, var_name, val)) },
                var.clone(),
                Box::new(substitute_variable(point, var_name, val))
            )
        },
        AST::Intg(expr, var, lower, upper) => {
            AST::Intg(
                if var == var_name { expr.clone() } else { Box::new(substitute_variable(expr, var_name, val)) },
                var.clone(),
                Box::new(substitute_variable(lower, var_name, val)),
                Box::new(substitute_variable(upper, var_name, val))
            )
        },
        AST::Sum(expr, var, start, end) => {
            AST::Sum(
                if var == var_name { expr.clone() } else { Box::new(substitute_variable(expr, var_name, val)) },
                var.clone(),
                Box::new(substitute_variable(start, var_name, val)),
                Box::new(substitute_variable(end, var_name, val))
            )
        },
        AST::Prod(expr, var, start, end) => {
            AST::Prod(
                if var == var_name { expr.clone() } else { Box::new(substitute_variable(expr, var_name, val)) },
                var.clone(),
                Box::new(substitute_variable(start, var_name, val)),
                Box::new(substitute_variable(end, var_name, val))
            )
        },
        AST::MatrixLiteral(rows) => {
            let mut new_rows = Vec::new();
            for row in rows {
                let mut new_row = Vec::new();
                for element in row {
                    new_row.push(substitute_variable(element, var_name, val));
                }
                new_rows.push(new_row);
            }
            AST::MatrixLiteral(new_rows)
        },
        AST::PolyReg(x, y, deg) => {
            AST::PolyReg(
                Box::new(substitute_variable(x, var_name, val)),
                Box::new(substitute_variable(y, var_name, val)),
                Box::new(substitute_variable(deg, var_name, val))
            )
        },
        AST::ProbFunc(func, args) => {
            let new_args = args.iter().map(|arg| substitute_variable(arg, var_name, val)).collect();
            AST::ProbFunc(func.clone(), new_args)
        }
    }
}

/// Evalúa una expresión matemática (ej. "sin(x)") sustituyendo la variable "x" por el valor especificado.
/// Retorna f64::NAN si la expresión es inválida o tiene errores.
#[no_mangle]
pub extern "C" fn evaluate_with_var_ffi(
    input: *const c_char,
    var_value: f64
) -> f64 {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if input.is_null() {
            return std::f64::NAN;
        }

        let c_str = unsafe { CStr::from_ptr(input) };
        let input_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return std::f64::NAN,
        };

        let tokens = match lex(input_str) {
            Ok(t) => t,
            Err(_) => return std::f64::NAN,
        };

        let (ast, _rest) = match parse(&tokens) {
            Ok((tree, remaining)) => {
                if !remaining.is_empty() {
                    return std::f64::NAN;
                }
                (tree, remaining)
            },
            Err(_) => return std::f64::NAN,
        };

        let substituted = substitute_variable(&ast, "x", var_value);
        let substituted = substitute_variable(&substituted, "y", var_value);

        match evaluate_infix(&substituted) {
            Ok(val) => val.to_scalar().unwrap_or(std::f64::NAN),
            Err(_) => std::f64::NAN,
        }
    }));

    match panic_res {
        Ok(val) => val,
        Err(_) => std::f64::NAN,
    }
}

/// Evalúa una expresión matemática paramétrica sustituyendo la variable "t" por el valor especificado.
/// Retorna f64::NAN si la expresión es inválida o tiene errores.
#[no_mangle]
pub extern "C" fn evaluate_parametric_ffi(
    input: *const c_char,
    t_val: f64
) -> f64 {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if input.is_null() {
            return std::f64::NAN;
        }

        let c_str = unsafe { CStr::from_ptr(input) };
        let input_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return std::f64::NAN,
        };

        let tokens = match lex(input_str) {
            Ok(t) => t,
            Err(_) => return std::f64::NAN,
        };

        let (ast, _rest) = match parse(&tokens) {
            Ok((tree, remaining)) => {
                if !remaining.is_empty() {
                    return std::f64::NAN;
                }
                (tree, remaining)
            },
            Err(_) => return std::f64::NAN,
        };

        let substituted = substitute_variable(&ast, "t", t_val);

        match evaluate_infix(&substituted) {
            Ok(val) => val.to_scalar().unwrap_or(std::f64::NAN),
            Err(_) => std::f64::NAN,
        }
    }));

    match panic_res {
        Ok(val) => val,
        Err(_) => std::f64::NAN,
    }
}

/// Evalúa una expresión matemática polar sustituyendo la variable "theta" (o "θ") por el valor especificado.
/// Retorna f64::NAN si la expresión es inválida o tiene errores.
#[no_mangle]
pub extern "C" fn evaluate_polar_ffi(
    input: *const c_char,
    theta_val: f64
) -> f64 {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if input.is_null() {
            return std::f64::NAN;
        }

        let c_str = unsafe { CStr::from_ptr(input) };
        let input_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return std::f64::NAN,
        };

        let tokens = match lex(input_str) {
            Ok(t) => t,
            Err(_) => return std::f64::NAN,
        };

        let (ast, _rest) = match parse(&tokens) {
            Ok((tree, remaining)) => {
                if !remaining.is_empty() {
                    return std::f64::NAN;
                }
                (tree, remaining)
            },
            Err(_) => return std::f64::NAN,
        };

        // Reemplazar tanto "theta" como "θ"
        let substituted = substitute_variable(&ast, "theta", theta_val);
        let substituted = substitute_variable(&substituted, "θ", theta_val);

        match evaluate_infix(&substituted) {
            Ok(val) => val.to_scalar().unwrap_or(std::f64::NAN),
            Err(_) => std::f64::NAN,
        }
    }));

    match panic_res {
        Ok(val) => val,
        Err(_) => std::f64::NAN,
    }
}

/// Detecta si una expresión es infija (1), posfija/RPN (2), o inválida/vacía (0).
#[no_mangle]
pub extern "C" fn detect_notation_ffi(input: *const c_char) -> u8 {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if input.is_null() {
            return 0;
        }
        
        let c_str = unsafe { CStr::from_ptr(input) };
        let input_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        if input_str.trim().is_empty() {
            return 0;
        }

        let tokens = match lex(input_str) {
            Ok(t) => t,
            Err(_) => return 0,
        };

        if tokens.is_empty() {
            return 0;
        }

        if is_postfix_expression(&tokens) {
            2
        } else {
            1
        }
    }));

    match panic_res {
        Ok(val) => val,
        Err(_) => 0,
    }
}

/// Calcula el espaciado adaptable de la cuadrícula de graficación a partir del rango.
#[no_mangle]
pub extern "C" fn calculate_grid_step_ffi(min_val: f64, max_val: f64) -> f64 {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let range = max_val - min_val;
        if range <= 0.0 {
            return 1.0;
        }

        let raw_step = range / 10.0;
        if raw_step <= 0.0 {
            return 1.0;
        }

        let exponent = raw_step.log10().floor();
        let power_of_10 = 10.0f64.powf(exponent);
        let ratio = raw_step / power_of_10;
        
        let mut step = power_of_10;
        if ratio >= 5.0 {
            step = 5.0 * power_of_10;
        } else if ratio >= 2.0 {
            step = 2.0 * power_of_10;
        }
        
        step
    }));

    match panic_res {
        Ok(val) => val,
        Err(_) => 1.0,
    }
}

#[no_mangle]
pub extern "C" fn set_angle_mode_ffi(degrees: u8) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        calculator::calculator::set_angle_mode(degrees);
    }));
}

#[no_mangle]
pub extern "C" fn get_last_result_ffi() -> f64 {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        calculator::calculator::get_last_result()
    }));
    match panic_res {
        Ok(v) => v,
        Err(_) => 0.0,
    }
}

#[no_mangle]
pub extern "C" fn clear_last_result_ffi() {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        calculator::calculator::clear_last_result();
    }));
}

/// Asigna un valor numérico real a una variable en la memoria global de la calculadora.
#[no_mangle]
pub extern "C" fn assign_variable_ffi(name: *const c_char, value: f64) -> u8 {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if name.is_null() {
            return 0;
        }
        let c_str = unsafe { CStr::from_ptr(name) };
        let name_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };
        crate::calculator::value::set_variable(name_str, crate::calculator::value::Value::Scalar(value));
        1
    }));
    match panic_res {
        Ok(v) => v,
        Err(_) => 0,
    }
}

/// Recupera el valor numérico real de una variable de la memoria global.
/// Retorna NaN si la variable no existe o no es de tipo real/escalar.
#[no_mangle]
pub extern "C" fn get_variable_ffi(name: *const c_char) -> f64 {
    let panic_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if name.is_null() {
            return std::f64::NAN;
        }
        let c_str = unsafe { CStr::from_ptr(name) };
        let name_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return std::f64::NAN,
        };
        match crate::calculator::value::get_variable(name_str) {
            Some(crate::calculator::value::Value::Scalar(v)) => v,
            Some(crate::calculator::value::Value::Complex(c)) if c.im == 0.0 => c.re,
            _ => std::f64::NAN,
        }
    }));
    match panic_res {
        Ok(v) => v,
        Err(_) => std::f64::NAN,
    }
}


/// Libera la memoria de una cadena de caracteres asignada por Rust (usando CString::into_raw).
#[no_mangle]
pub extern "C" fn free_string_ffi(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

#[cfg(test)]
mod ffi_tests {
    use super::*;

    #[test]
    fn test_process_expression_ffi() {
        let input = CString::new("2 + 3 * 4").unwrap();
        let res = process_expression_ffi(input.as_ptr());
        assert_eq!(res.is_success, 1);
        let val_c = unsafe { CStr::from_ptr(res.value) };
        assert_eq!(val_c.to_str().unwrap(), "14");
        free_string_ffi(res.value);
    }

    #[test]
    fn test_validate_expression_ffi() {
        let valid = CString::new("2 + 3").unwrap();
        let invalid = CString::new("2 + ").unwrap();
        assert_eq!(validate_expression_ffi(valid.as_ptr()), 1);
        assert_eq!(validate_expression_ffi(invalid.as_ptr()), 0);
    }

    #[test]
    fn test_convert_units_ffi_length() {
        let m = CString::new("m").unwrap();
        let km = CString::new("km").unwrap();
        let cat = CString::new("length").unwrap();
        
        let result = convert_units_ffi(1500.0, m.as_ptr(), km.as_ptr(), cat.as_ptr());
        assert!((result - 1.5).abs() < 1e-9);
    }

    #[test]
    fn test_convert_units_ffi_temperature() {
        let c = CString::new("C").unwrap();
        let f = CString::new("F").unwrap();
        let cat = CString::new("temperature").unwrap();
        
        let result = convert_units_ffi(100.0, c.as_ptr(), f.as_ptr(), cat.as_ptr());
        assert!((result - 212.0).abs() < 1e-9);
    }

    #[test]
    fn test_evaluate_with_var_ffi() {
        let expr = CString::new("sin(x)").unwrap();
        let result = evaluate_with_var_ffi(expr.as_ptr(), std::f64::consts::PI / 2.0);
        assert!((result - 1.0).abs() < 1e-9);

        let expr2 = CString::new("x * x - 4").unwrap();
        let result2 = evaluate_with_var_ffi(expr2.as_ptr(), 3.0);
        assert!((result2 - 5.0).abs() < 1e-9);

        let expr_invalid = CString::new("x +").unwrap();
        let result_invalid = evaluate_with_var_ffi(expr_invalid.as_ptr(), 2.0);
        assert!(result_invalid.is_nan());
    }

    #[test]
    fn test_detect_notation_ffi() {
        let infix = CString::new("2 + 3 * 4").unwrap();
        let rpn = CString::new("2 3 4 * +").unwrap();
        let empty = CString::new("   ").unwrap();
        
        assert_eq!(detect_notation_ffi(infix.as_ptr()), 1);
        assert_eq!(detect_notation_ffi(rpn.as_ptr()), 2);
        assert_eq!(detect_notation_ffi(empty.as_ptr()), 0);
    }

    #[test]
    fn test_calculate_grid_step_ffi() {
        let step1 = calculate_grid_step_ffi(-10.0, 10.0);
        assert!((step1 - 2.0).abs() < 1e-9); // Range 20, step should be 2.0
        
        let step2 = calculate_grid_step_ffi(0.0, 100.0);
        assert!((step2 - 10.0).abs() < 1e-9); // Range 100, step should be 10.0
    }

    #[test]
    fn test_evaluate_parametric_and_polar_ffi() {
        let expr_t = CString::new("t * t - 2").unwrap();
        let result_t = evaluate_parametric_ffi(expr_t.as_ptr(), 3.0);
        assert!((result_t - 7.0).abs() < 1e-9);

        let expr_theta = CString::new("3 * cos(theta)").unwrap();
        let result_theta = evaluate_polar_ffi(expr_theta.as_ptr(), 0.0);
        assert!((result_theta - 3.0).abs() < 1e-9);

        let expr_theta_sym = CString::new("3 * cos(θ)").unwrap();
        let result_theta_sym = evaluate_polar_ffi(expr_theta_sym.as_ptr(), 0.0);
        assert!((result_theta_sym - 3.0).abs() < 1e-9);
    }
}
