use crate::calculator::ast::{AST, Operator, Function, Function2, ProbFunction};
use crate::calculator::calculator::CalculatorError;
use crate::calculator::token::Token;
use crate::calculator::parser::parse;
use crate::calculator::value::Value;
use num_complex::Complex;
use statrs::distribution::{Normal, Binomial, Poisson, Continuous, ContinuousCDF, Discrete, DiscreteCDF};

fn factorial_f64(n: f64) -> Result<f64, CalculatorError> {
    if n < 0.0 || n.fract() != 0.0 {
        return Err(CalculatorError::InvalidExpression);
    }
    let limit = n as u64;
    if limit > 170 {
        // En lugar de error, devolver infinito como las calculadoras científicas
        return Ok(f64::INFINITY);
    }
    let mut res = 1.0;
    for i in 1..=limit {
        res *= i as f64;
    }
    Ok(res)
}

fn gcd_f64(a: f64, b: f64) -> Result<f64, CalculatorError> {
    if !a.is_finite() || !b.is_finite() || a.fract() != 0.0 || b.fract() != 0.0 {
        return Err(CalculatorError::InvalidExpression);
    }
    if a.abs() > u64::MAX as f64 || b.abs() > u64::MAX as f64 {
        return Err(CalculatorError::InvalidExpression);
    }
    let mut x = a.abs() as u64;
    let mut y = b.abs() as u64;
    while y != 0 {
        let tmp = y;
        y = x % y;
        x = tmp;
    }
    Ok(x as f64)
}

fn lcm_f64(a: f64, b: f64) -> Result<f64, CalculatorError> {
    if !a.is_finite() || !b.is_finite() || a.fract() != 0.0 || b.fract() != 0.0 {
        return Err(CalculatorError::InvalidExpression);
    }
    if a.abs() > i64::MAX as f64 || b.abs() > i64::MAX as f64 {
        return Err(CalculatorError::InvalidExpression);
    }
    let x = a.abs() as i64;
    let y = b.abs() as i64;
    use num_integer::Integer;
    Ok(x.lcm(&y) as f64)
}

fn root_f64(x: f64, n: f64) -> Result<f64, CalculatorError> {
    if n == 0.0 {
        return Err(CalculatorError::DivisionByZero);
    }
    if x < 0.0 {
        if n.fract() == 0.0 && (n as i64) % 2 != 0 {
            Ok(-(-x).powf(1.0 / n))
        } else {
            Ok(std::f64::NAN)
        }
    } else {
        Ok(x.powf(1.0 / n))
    }
}

fn n_choose_r(n: f64, r: f64) -> Result<f64, CalculatorError> {
    if n < 0.0 || r < 0.0 || n.fract() != 0.0 || r.fract() != 0.0 {
        return Err(CalculatorError::InvalidExpression);
    }
    if r > n {
        return Err(CalculatorError::InvalidExpression);
    }
    let n_u = n as u64;
    let mut r_u = r as u64;
    if r_u > n_u / 2 {
        r_u = n_u - r_u;
    }
    let mut result = 1.0;
    for i in 0..r_u {
        result *= (n_u - i) as f64;
        result /= (i + 1) as f64;
    }
    if !result.is_finite() {
        return Err(CalculatorError::InvalidExpression);
    }
    Ok(result)
}

fn n_permute_r(n: f64, r: f64) -> Result<f64, CalculatorError> {
    if n < 0.0 || r < 0.0 || n.fract() != 0.0 || r.fract() != 0.0 {
        return Err(CalculatorError::InvalidExpression);
    }
    if r > n {
        return Err(CalculatorError::InvalidExpression);
    }
    let n_u = n as u64;
    let r_u = r as u64;
    let mut result = 1.0;
    for i in 0..r_u {
        result *= (n_u - i) as f64;
    }
    if !result.is_finite() {
        return Err(CalculatorError::InvalidExpression);
    }
    Ok(result)
}

fn extract_coefficients_and_constants(ast: &AST, target_var: &str) -> Result<(f64, f64), CalculatorError> {
    match ast {
        AST::Num(n) => Ok((0.0, *n)),
        AST::Var(name) => {
            if name == target_var {
                Ok((1.0, 0.0))
            } else if let Some(Value::Scalar(v)) = crate::calculator::value::get_variable(name) {
                Ok((0.0, v))
            } else {
                Err(CalculatorError::InvalidExpression)
            }
        },
        AST::BinOp(lhs, op, rhs) => {
            let (lhs_coeff, lhs_const) = extract_coefficients_and_constants(lhs, target_var)?;
            let (rhs_coeff, rhs_const) = extract_coefficients_and_constants(rhs, target_var)?;

            match op {
                Operator::Add => Ok((lhs_coeff + rhs_coeff, lhs_const + rhs_const)),
                Operator::Sub => Ok((lhs_coeff - rhs_coeff, lhs_const - rhs_const)),
                Operator::Mul => {
                    if lhs_coeff == 0.0 {
                        Ok((rhs_coeff * lhs_const, rhs_const * lhs_const))
                    } else if rhs_coeff == 0.0 {
                        Ok((lhs_coeff * rhs_const, lhs_const * rhs_const))
                    } else {
                        Err(CalculatorError::InvalidExpression)
                    }
                },
                Operator::Div => {
                    if rhs_coeff != 0.0 {
                        Err(CalculatorError::InvalidExpression)
                    } else if rhs_const == 0.0 {
                        Err(CalculatorError::DivisionByZero)
                    } else {
                        Ok((lhs_coeff / rhs_const, lhs_const / rhs_const))
                    }
                },
                Operator::Power | Operator::Mod | Operator::Percent => Err(CalculatorError::InvalidExpression),
            }
        },
        AST::Const(c) => Ok((0.0, *c)),
        _ => return Err(CalculatorError::UnexpectedToken),
    }
}

pub(crate) fn solve_equation(tokens: &[Token], target_var: &str) -> Result<f64, CalculatorError> {
    let equal_pos = tokens.iter().position(|t| *t == Token::Equal)
        .ok_or(CalculatorError::ParseError)?;

    let (left_tokens, right_tokens) = tokens.split_at(equal_pos);
    let right_tokens = &right_tokens[1..];

    let (left_ast, _) = parse(left_tokens)?;
    let (right_ast, _) = parse(right_tokens)?;

    let (left_coefficient, left_constant) = extract_coefficients_and_constants(&left_ast, target_var)?;
    let (right_coefficient, right_constant) = extract_coefficients_and_constants(&right_ast, target_var)?;

    let a = left_coefficient - right_coefficient;
    let b = right_constant - left_constant;

    if a == 0.0 {
        return Err(CalculatorError::InvalidExpression);
    }

    Ok(b / a)
}

pub(crate) fn evaluate_infix(ast: &AST) -> Result<Value, CalculatorError> {
    match ast {
        AST::Num(n) => Ok(Value::Scalar(*n)),
        AST::Var(name) => {
            if name == "__ANS__" {
                Ok(Value::Scalar(crate::calculator::calculator::get_last_result()))
            } else if let Some(val) = crate::calculator::value::get_variable(name) {
                Ok(val)
            } else {
                Err(CalculatorError::InvalidExpression)
            }
        },
        AST::BinOp(lhs, op, rhs) => {
            let lhs_val = evaluate_infix(lhs)?;
            let rhs_val = evaluate_infix(rhs)?;
            match op {
                Operator::Add => lhs_val.add(rhs_val),
                Operator::Sub => lhs_val.sub(rhs_val),
                Operator::Mul => lhs_val.mul(rhs_val),
                Operator::Div => lhs_val.div(rhs_val),
                Operator::Mod => lhs_val.rem(rhs_val),
                Operator::Percent => {
                    let val_100 = Value::Scalar(100.0);
                    lhs_val.mul(rhs_val)?.div(val_100)
                }
                Operator::Power => lhs_val.pow(rhs_val),
            }
        },
        AST::Func(func, arg) => {
            let arg_val = evaluate_infix(arg)?;
            apply_function(func, arg_val)
        },
        AST::Func2(func, lhs, rhs) => {
            let lhs_val = evaluate_infix(lhs)?;
            let rhs_val = evaluate_infix(rhs)?;
            apply_function2(func, lhs_val, rhs_val)
        },
        AST::Const(c) => Ok(Value::Scalar(*c)),
        AST::LogBase(base, expr) => {
            let expr_val = evaluate_infix(expr)?;
            let val = expr_val.to_scalar()?;
            if val <= 0.0 {
                return Err(CalculatorError::InvalidExpression);
            }
            if *base <= 0.0 || *base == 1.0 {
                return Err(CalculatorError::InvalidExpression);
            }
            Ok(Value::Scalar(val.log(*base)))
        },
        AST::Deriv(expr, var, point) => {
            let point_val = evaluate_infix(point)?.to_scalar()?;
            let h = 1e-5;
            let f_p2 = eval_at(expr, var, point_val + 2.0 * h)?;
            let f_p1 = eval_at(expr, var, point_val + h)?;
            let f_m1 = eval_at(expr, var, point_val - h)?;
            let f_m2 = eval_at(expr, var, point_val - 2.0 * h)?;
            let deriv_val = (-f_p2 + 8.0 * f_p1 - 8.0 * f_m1 + f_m2) / (12.0 * h);
            Ok(Value::Scalar(deriv_val))
        }
        AST::Intg(expr, var, lower, upper) => {
            let a = evaluate_infix(lower)?.to_scalar()?;
            let b = evaluate_infix(upper)?.to_scalar()?;
            let n = 1000;
            let h = (b - a) / (n as f64);
            let mut sum_even = 0.0;
            let mut sum_odd = 0.0;
            
            let f_a = eval_at(expr, var, a)?;
            let f_b = eval_at(expr, var, b)?;
            
            for i in 1..n {
                let x = a + (i as f64) * h;
                let y = eval_at(expr, var, x)?;
                if i % 2 == 0 {
                    sum_even += y;
                } else {
                    sum_odd += y;
                }
            }
            let integral = (h / 3.0) * (f_a + f_b + 2.0 * sum_even + 4.0 * sum_odd);
            Ok(Value::Scalar(integral))
        }
        AST::Sum(expr, var, start, end) => {
            let start_val = evaluate_infix(start)?.to_scalar()?.round() as i64;
            let end_val = evaluate_infix(end)?.to_scalar()?.round() as i64;
            let mut sum = Value::Scalar(0.0);
            for i in start_val..=end_val {
                let orig_val = crate::calculator::value::get_variable(var);
                crate::calculator::value::set_variable(var, Value::Scalar(i as f64));
                let term = evaluate_infix(expr);
                if let Some(orig) = orig_val {
                    crate::calculator::value::set_variable(var, orig);
                } else {
                    if let Ok(mut g) = crate::calculator::value::get_variables().lock() {
                        g.remove(var);
                    }
                }
                sum = sum.add(term?)?;
            }
            Ok(sum)
        }
        AST::Prod(expr, var, start, end) => {
            let start_val = evaluate_infix(start)?.to_scalar()?.round() as i64;
            let end_val = evaluate_infix(end)?.to_scalar()?.round() as i64;
            let mut prod = Value::Scalar(1.0);
            for i in start_val..=end_val {
                let orig_val = crate::calculator::value::get_variable(var);
                crate::calculator::value::set_variable(var, Value::Scalar(i as f64));
                let term = evaluate_infix(expr);
                if let Some(orig) = orig_val {
                    crate::calculator::value::set_variable(var, orig);
                } else {
                    if let Ok(mut g) = crate::calculator::value::get_variables().lock() {
                        g.remove(var);
                    }
                }
                prod = prod.mul(term?)?;
            }
            Ok(prod)
        },
        AST::MatrixLiteral(rows) => {
            if rows.is_empty() {
                return Ok(Value::Vector(Vec::new()));
            }
            if rows.len() == 1 {
                let mut vec = Vec::new();
                for element in &rows[0] {
                    let val = evaluate_infix(element)?.to_scalar()?;
                    vec.push(val);
                }
                Ok(Value::Vector(vec))
            } else {
                let nrows = rows.len();
                let ncols = rows[0].len();
                let mut data = Vec::new();
                for row in rows {
                    for element in row {
                        let val = evaluate_infix(element)?.to_scalar()?;
                        data.push(val);
                    }
                }
                let matrix = nalgebra::DMatrix::from_row_slice(nrows, ncols, &data);
                Ok(Value::Matrix(matrix))
            }
        },
        AST::PolyReg(x_ast, y_ast, deg_ast) => {
            let x_val = evaluate_infix(x_ast)?;
            let y_val = evaluate_infix(y_ast)?;
            let deg_val = evaluate_infix(deg_ast)?.to_scalar()?;
            
            match (x_val, y_val) {
                (Value::Vector(x), Value::Vector(y)) => {
                    if x.len() != y.len() || x.is_empty() {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let d = deg_val.round() as usize;
                    if d < 1 || x.len() < d + 1 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let n = x.len();
                    let mut v_data = Vec::new();
                    for &xi in &x {
                        for power in (0..=d).rev() {
                            v_data.push(xi.powi(power as i32));
                        }
                    }
                    let v = nalgebra::DMatrix::from_row_slice(n, d + 1, &v_data);
                    let y_vec = nalgebra::DVector::from_vec(y.clone());
                    let vt = v.transpose();
                    let vtv = &vt * &v;
                    let vty = &vt * y_vec;
                    let decomp = vtv.lu();
                    let a_coeff = decomp.solve(&vty).ok_or(CalculatorError::InvalidExpression)?;
                    
                    let mean_y = y.iter().sum::<f64>() / (n as f64);
                    let ss_tot = y.iter().map(|yi| (yi - mean_y).powi(2)).sum::<f64>();
                    let y_pred = &v * &a_coeff;
                    let ss_res = y.iter().zip(y_pred.iter()).map(|(yi, ypi)| (yi - ypi).powi(2)).sum::<f64>();
                    let r_squared = if ss_tot == 0.0 {
                        1.0
                    } else {
                        1.0 - (ss_res / ss_tot)
                    };
                    
                    let mut res_vec = a_coeff.as_slice().to_vec();
                    res_vec.push(r_squared);
                    Ok(Value::Vector(res_vec))
                }
                _ => Err(CalculatorError::InvalidExpression),
            }
        },
        AST::ProbFunc(func, args) => {
            let mut eval_args = Vec::new();
            for arg in args {
                eval_args.push(evaluate_infix(arg)?.to_scalar()?);
            }
            match func {
                ProbFunction::Rand => {
                    let r: f64 = rand::random::<f64>();
                    if eval_args.is_empty() {
                        Ok(Value::Scalar(r))
                    } else if eval_args.len() == 1 {
                        Ok(Value::Scalar(r * eval_args[0]))
                    } else if eval_args.len() == 2 {
                        Ok(Value::Scalar(eval_args[0] + r * (eval_args[1] - eval_args[0])))
                    } else {
                        Err(CalculatorError::InvalidExpression)
                    }
                }
                ProbFunction::NormPdf => {
                    let (x, mean, std) = if eval_args.len() == 1 {
                        (eval_args[0], 0.0, 1.0)
                    } else if eval_args.len() == 3 {
                        (eval_args[0], eval_args[1], eval_args[2])
                    } else {
                        return Err(CalculatorError::InvalidExpression);
                    };
                    if std <= 0.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let dist = Normal::new(mean, std).map_err(|_| CalculatorError::InvalidExpression)?;
                    Ok(Value::Scalar(dist.pdf(x)))
                }
                ProbFunction::NormCdf => {
                    let (x, mean, std) = if eval_args.len() == 1 {
                        (eval_args[0], 0.0, 1.0)
                    } else if eval_args.len() == 3 {
                        (eval_args[0], eval_args[1], eval_args[2])
                    } else {
                        return Err(CalculatorError::InvalidExpression);
                    };
                    if std <= 0.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let dist = Normal::new(mean, std).map_err(|_| CalculatorError::InvalidExpression)?;
                    Ok(Value::Scalar(dist.cdf(x)))
                }
                ProbFunction::BinoPdf => {
                    if eval_args.len() != 3 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let k = eval_args[0];
                    let n = eval_args[1];
                    let p = eval_args[2];
                    if k < 0.0 || n < 0.0 || k.fract() != 0.0 || n.fract() != 0.0 || k > n || p < 0.0 || p > 1.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let dist = Binomial::new(p, n as u64).map_err(|_| CalculatorError::InvalidExpression)?;
                    Ok(Value::Scalar(dist.pmf(k as u64)))
                }
                ProbFunction::BinoCdf => {
                    if eval_args.len() != 3 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let k = eval_args[0];
                    let n = eval_args[1];
                    let p = eval_args[2];
                    if k < 0.0 || n < 0.0 || k.fract() != 0.0 || n.fract() != 0.0 || k > n || p < 0.0 || p > 1.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let dist = Binomial::new(p, n as u64).map_err(|_| CalculatorError::InvalidExpression)?;
                    Ok(Value::Scalar(dist.cdf(k as u64)))
                }
                ProbFunction::PoissPdf => {
                    if eval_args.len() != 2 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let k = eval_args[0];
                    let lambda = eval_args[1];
                    if k < 0.0 || k.fract() != 0.0 || lambda <= 0.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let dist = Poisson::new(lambda).map_err(|_| CalculatorError::InvalidExpression)?;
                    Ok(Value::Scalar(dist.pmf(k as u64)))
                }
                ProbFunction::PoissCdf => {
                    if eval_args.len() != 2 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let k = eval_args[0];
                    let lambda = eval_args[1];
                    if k < 0.0 || k.fract() != 0.0 || lambda <= 0.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let dist = Poisson::new(lambda).map_err(|_| CalculatorError::InvalidExpression)?;
                    Ok(Value::Scalar(dist.cdf(k as u64)))
                }
            }
        }
    }
}

fn eval_at(expr: &AST, var_name: &str, val: f64) -> Result<f64, CalculatorError> {
    let orig_val = crate::calculator::value::get_variable(var_name);
    crate::calculator::value::set_variable(var_name, Value::Scalar(val));
    let res = evaluate_infix(expr);
    if let Some(orig) = orig_val {
        crate::calculator::value::set_variable(var_name, orig);
    } else {
        if let Ok(mut g) = crate::calculator::value::get_variables().lock() {
            g.remove(var_name);
        }
    }
    res.and_then(|v| v.to_scalar())
}

fn to_radians(val: f64, mode: u8) -> f64 {
    match mode {
        1 => val.to_radians(), // Degrees -> Radians
        2 => val * std::f64::consts::PI / 200.0, // Gradians -> Radians
        _ => val, // Radians -> Radians
    }
}

fn from_radians(val: f64, mode: u8) -> f64 {
    match mode {
        1 => val.to_degrees(), // Radians -> Degrees
        2 => val * 200.0 / std::f64::consts::PI, // Radians -> Gradians
        _ => val, // Radians -> Radians
    }
}

fn apply_function(func: &Function, arg_val: Value) -> Result<Value, CalculatorError> {
    let angle_mode = crate::calculator::calculator::get_angle_mode();
    match arg_val {
        Value::Scalar(arg_f64) => {
            let res_f64 = match func {
                Function::Log => {
                    if arg_f64 <= 0.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    arg_f64.log10()
                },
                Function::Ln => {
                    if arg_f64 <= 0.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    arg_f64.ln()
                },
                Function::Sin => {
                    let r = to_radians(arg_f64, angle_mode);
                    r.sin()
                },
                Function::Cos => {
                    let r = to_radians(arg_f64, angle_mode);
                    r.cos()
                },
                Function::Tan => {
                    let r = to_radians(arg_f64, angle_mode);
                    r.tan()
                },
                Function::Ctan => {
                    let r = to_radians(arg_f64, angle_mode);
                    let t = r.tan();
                    if t == 0.0 {
                        return Err(CalculatorError::DivisionByZero);
                    } else {
                        1.0 / t
                    }
                },
                Function::Sqrt => {
                    if arg_f64 < 0.0 {
                        // sqrt(-n) = 0 + sqrt(n)i
                        return Ok(Value::Complex(num_complex::Complex::new(0.0, (-arg_f64).sqrt())));
                    }
                    arg_f64.sqrt()
                },
                Function::Abs => arg_f64.abs(),
                Function::Asin => {
                    if arg_f64 < -1.0 || arg_f64 > 1.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let v = arg_f64.asin();
                    from_radians(v, angle_mode)
                },
                Function::Acos => {
                    if arg_f64 < -1.0 || arg_f64 > 1.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let v = arg_f64.acos();
                    from_radians(v, angle_mode)
                },
                Function::Atan => {
                    let v = arg_f64.atan();
                    from_radians(v, angle_mode)
                },
                Function::Sinh => arg_f64.sinh(),
                Function::Cosh => arg_f64.cosh(),
                Function::Tanh => arg_f64.tanh(),
                Function::Asinh => arg_f64.asinh(),
                Function::Acosh => {
                    if arg_f64 < 1.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    arg_f64.acosh()
                },
                Function::Atanh => {
                    if arg_f64 <= -1.0 || arg_f64 >= 1.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    arg_f64.atanh()
                },
                Function::Fact => factorial_f64(arg_f64)?,
                Function::Floor => arg_f64.floor(),
                Function::Ceil => arg_f64.ceil(),
                Function::Round => arg_f64.round(),
                Function::Trunc => arg_f64.trunc(),
                Function::Int => arg_f64.trunc(),
                Function::Fract => arg_f64.fract(),
                Function::Cbrt => arg_f64.cbrt(),
                Function::Exp => arg_f64.exp(),
                Function::Re => arg_f64,
                Function::Im => 0.0,
                Function::Conj => arg_f64,
                Function::Arg => {
                    if arg_f64 >= 0.0 {
                        0.0
                    } else {
                        std::f64::consts::PI
                    }
                }
                Function::Mean => arg_f64,
                Function::Median => arg_f64,
                Function::Var => 0.0,
                Function::Std => 0.0,
                Function::Det => arg_f64,
                Function::Inv => {
                    if arg_f64 == 0.0 {
                        return Err(CalculatorError::DivisionByZero);
                    }
                    1.0 / arg_f64
                }
                Function::Transpose => arg_f64,
                Function::Sum => arg_f64,          // sum(scalar) = scalar
                Function::Sort => arg_f64,          // sort(scalar) = scalar
                Function::Tr => arg_f64,            // tr(scalar) = scalar
                Function::MinVec => arg_f64,        // min(scalar) = scalar
                Function::MaxVec => arg_f64,        // max(scalar) = scalar
            };
            Ok(Value::Scalar(res_f64))
        }
        Value::Complex(c) => {
            match func {
                Function::Re => Ok(Value::Scalar(c.re)),
                Function::Im => Ok(Value::Scalar(c.im)),
                Function::Conj => Ok(Value::Complex(c.conj())),
                Function::Arg => Ok(Value::Scalar(c.arg())),
                Function::Exp => Ok(Value::Complex(c.exp())),
                Function::Sin => Ok(Value::Complex(c.sin())),
                Function::Cos => Ok(Value::Complex(c.cos())),
                Function::Tan => Ok(Value::Complex(c.tan())),
                Function::Ctan => {
                    let t = c.tan();
                    if t.norm() == 0.0 {
                        Err(CalculatorError::DivisionByZero)
                    } else {
                        Ok(Value::Complex(Complex::new(1.0, 0.0) / t))
                    }
                }
                Function::Sqrt => Ok(Value::Complex(c.sqrt())),
                Function::Abs => Ok(Value::Scalar(c.norm())),
                Function::Ln => Ok(Value::Complex(c.ln())),
                Function::Log => Ok(Value::Complex(c.ln() / 10.0_f64.ln())),
                Function::Sinh => Ok(Value::Complex(c.sinh())),
                Function::Cosh => Ok(Value::Complex(c.cosh())),
                Function::Tanh => Ok(Value::Complex(c.tanh())),
                Function::Asinh => Ok(Value::Complex(c.asinh())),
                Function::Acosh => Ok(Value::Complex(c.acosh())),
                Function::Atanh => Ok(Value::Complex(c.atanh())),
                Function::Asin => Ok(Value::Complex(c.asin())),
                Function::Acos => Ok(Value::Complex(c.acos())),
                Function::Atan => Ok(Value::Complex(c.atan())),
                Function::Det => Ok(Value::Complex(c)),
                Function::Inv => {
                    if c.norm() == 0.0 {
                        Err(CalculatorError::DivisionByZero)
                    } else {
                        Ok(Value::Complex(Complex::new(1.0, 0.0) / c))
                    }
                }
                Function::Transpose => Ok(Value::Complex(c)),
                _ => Err(CalculatorError::InvalidExpression),
            }
        }
        Value::Vector(v) => {
            match func {
                Function::Mean => {
                    if v.is_empty() {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let sum: f64 = v.iter().sum();
                    Ok(Value::Scalar(sum / (v.len() as f64)))
                }
                Function::Median => {
                    if v.is_empty() {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let mut sorted = v.clone();
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let mid = sorted.len() / 2;
                    let median = if sorted.len() % 2 != 0 {
                        sorted[mid]
                    } else {
                        (sorted[mid - 1] + sorted[mid]) / 2.0
                    };
                    Ok(Value::Scalar(median))
                }
                Function::Var => {
                    if v.len() < 2 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let mean = v.iter().sum::<f64>() / (v.len() as f64);
                    let variance = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / ((v.len() - 1) as f64);
                    Ok(Value::Scalar(variance))
                }
                Function::Std => {
                    if v.len() < 2 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let mean = v.iter().sum::<f64>() / (v.len() as f64);
                    let variance = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / ((v.len() - 1) as f64);
                    Ok(Value::Scalar(variance.sqrt()))
                }
                Function::Transpose => {
                    Ok(Value::Matrix(nalgebra::DMatrix::from_row_slice(1, v.len(), &v)))
                }
                Function::Sum => {
                    // sum([1,2,3,4]) = 10
                    Ok(Value::Scalar(v.iter().sum()))
                }
                Function::MinVec => {
                    if v.is_empty() { return Err(CalculatorError::InvalidExpression); }
                    Ok(Value::Scalar(v.iter().cloned().fold(f64::INFINITY, f64::min)))
                }
                Function::MaxVec => {
                    if v.is_empty() { return Err(CalculatorError::InvalidExpression); }
                    Ok(Value::Scalar(v.iter().cloned().fold(f64::NEG_INFINITY, f64::max)))
                }
                Function::Sort => {
                    // sort([3,1,2]) = [1,2,3]
                    let mut sorted = v.clone();
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    Ok(Value::Vector(sorted))
                }
                _ => Err(CalculatorError::InvalidExpression),
            }
        }
        Value::Matrix(m) => {
            match func {
                Function::Det => {
                    if m.nrows() != m.ncols() {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    Ok(Value::Scalar(m.determinant()))
                }
                Function::Inv => {
                    if m.nrows() != m.ncols() {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let inv_matrix = m.try_inverse().ok_or(CalculatorError::InvalidExpression)?;
                    Ok(Value::Matrix(inv_matrix))
                }
                Function::Transpose => {
                    Ok(Value::Matrix(m.transpose()))
                }
                Function::Tr => {
                    // Traza: suma de elementos diagonales
                    if m.nrows() != m.ncols() {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    Ok(Value::Scalar(m.trace()))
                }
                _ => Err(CalculatorError::InvalidExpression),
            }
        }
    }
}

fn apply_function2(func: &Function2, lhs_val: Value, rhs_val: Value) -> Result<Value, CalculatorError> {
    let angle_mode = crate::calculator::calculator::get_angle_mode();
    match (lhs_val, rhs_val) {
        (Value::Scalar(lhs_f64), Value::Scalar(rhs_f64)) => {
            let res_val = match func {
                Function2::Min => Value::Scalar(lhs_f64.min(rhs_f64)),
                Function2::Max => Value::Scalar(lhs_f64.max(rhs_f64)),
                Function2::Mod => {
                    if rhs_f64 == 0.0 {
                        return Err(CalculatorError::DivisionByZero);
                    }
                    Value::Scalar(lhs_f64 - (lhs_f64 / rhs_f64).floor() * rhs_f64)
                }
                Function2::Gcd => Value::Scalar(gcd_f64(lhs_f64, rhs_f64)?),
                Function2::Lcm => Value::Scalar(lcm_f64(lhs_f64, rhs_f64)?),
                Function2::Ncr => Value::Scalar(n_choose_r(lhs_f64, rhs_f64)?),
                Function2::Npr => Value::Scalar(n_permute_r(lhs_f64, rhs_f64)?),
                Function2::Root => Value::Scalar(root_f64(lhs_f64, rhs_f64)?),
                Function2::Polar => {
                    let theta_rad = to_radians(rhs_f64, angle_mode);
                    Value::Complex(Complex::from_polar(lhs_f64, theta_rad))
                }
                Function2::LogBase => {
                    if lhs_f64 <= 0.0 || rhs_f64 <= 0.0 || rhs_f64 == 1.0 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    Value::Scalar(lhs_f64.log(rhs_f64))
                }
                _ => return Err(CalculatorError::InvalidExpression),
            };
            Ok(res_val)
        }
        (Value::Vector(x), Value::Vector(y)) => {
            match func {
                Function2::Cov => {
                    if x.len() != y.len() || x.len() < 2 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let n = x.len() as f64;
                    let mean_x = x.iter().sum::<f64>() / n;
                    let mean_y = y.iter().sum::<f64>() / n;
                    let cov = x.iter().zip(y.iter()).map(|(xi, yi)| (xi - mean_x) * (yi - mean_y)).sum::<f64>() / (n - 1.0);
                    Ok(Value::Scalar(cov))
                }
                Function2::Corr => {
                    if x.len() != y.len() || x.len() < 2 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let n = x.len() as f64;
                    let mean_x = x.iter().sum::<f64>() / n;
                    let mean_y = y.iter().sum::<f64>() / n;
                    let cov = x.iter().zip(y.iter()).map(|(xi, yi)| (xi - mean_x) * (yi - mean_y)).sum::<f64>() / (n - 1.0);
                    let var_x = x.iter().map(|xi| (xi - mean_x).powi(2)).sum::<f64>() / (n - 1.0);
                    let var_y = y.iter().map(|yi| (yi - mean_y).powi(2)).sum::<f64>() / (n - 1.0);
                    let corr = if var_x == 0.0 || var_y == 0.0 {
                        0.0
                    } else {
                        cov / (var_x.sqrt() * var_y.sqrt())
                    };
                    Ok(Value::Scalar(corr))
                }
                Function2::LinReg => {
                    if x.len() != y.len() || x.len() < 2 {
                        return Err(CalculatorError::InvalidExpression);
                    }
                    let n = x.len() as f64;
                    let mean_x = x.iter().sum::<f64>() / n;
                    let mean_y = y.iter().sum::<f64>() / n;
                    let cov = x.iter().zip(y.iter()).map(|(xi, yi)| (xi - mean_x) * (yi - mean_y)).sum::<f64>() / (n - 1.0);
                    let var_x = x.iter().map(|xi| (xi - mean_x).powi(2)).sum::<f64>() / (n - 1.0);
                    let var_y = y.iter().map(|yi| (yi - mean_y).powi(2)).sum::<f64>() / (n - 1.0);
                    let m = if var_x == 0.0 { 0.0 } else { cov / var_x };
                    let b = mean_y - m * mean_x;
                    let corr = if var_x == 0.0 || var_y == 0.0 {
                        0.0
                    } else {
                        cov / (var_x.sqrt() * var_y.sqrt())
                    };
                    Ok(Value::Vector(vec![m, b, corr, corr * corr]))
                }
                _ => Err(CalculatorError::InvalidExpression),
            }
        }
        _ => Err(CalculatorError::InvalidExpression),
    }
}


pub(crate) fn evaluate_postfix(tokens: &[Token]) -> Result<Value, CalculatorError> {
    let mut stack: Vec<Value> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(n) => stack.push(Value::Scalar(*n)),
            Token::Pi => stack.push(Value::Scalar(std::f64::consts::PI)),
            Token::E => stack.push(Value::Scalar(std::f64::consts::E)),
            Token::Tau => stack.push(Value::Scalar(std::f64::consts::TAU)),
            Token::Phi => stack.push(Value::Scalar(1.6180339887498949_f64)),
            Token::Sqrt2 => stack.push(Value::Scalar(std::f64::consts::SQRT_2)),
            Token::ConstC => stack.push(Value::Scalar(299792458.0)),
            Token::ConstH => stack.push(Value::Scalar(6.62607015e-34)),
            Token::ConstG => stack.push(Value::Scalar(6.6743e-11)),
            Token::Ans => stack.push(Value::Scalar(crate::calculator::calculator::get_last_result())),
            Token::Variable(name) => {
                if let Some(val) = crate::calculator::value::get_variable(name) {
                    stack.push(val);
                } else {
                    return Err(CalculatorError::InvalidExpression);
                }
            }
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Mod | Token::Percent | Token::Power => {
                if stack.len() < 2 {
                    return Err(CalculatorError::InvalidExpression);
                }
                let rhs = stack.pop().unwrap();
                let lhs = stack.pop().unwrap();
                let result = match token {
                    Token::Plus => lhs.add(rhs)?,
                    Token::Minus => lhs.sub(rhs)?,
                    Token::Multiply => lhs.mul(rhs)?,
                    Token::Divide => lhs.div(rhs)?,
                    Token::Mod => lhs.rem(rhs)?,
                    Token::Percent => lhs.mul(rhs)?.div(Value::Scalar(100.0))?,
                    Token::Power => lhs.pow(rhs)?,
                    _ => unreachable!(),
                };
                stack.push(result);
            },
            Token::Log | Token::Ln | Token::Sin | Token::Cos | Token::Tan | Token::Ctan | Token::LogBase(_) | Token::Sqrt | Token::Abs | Token::Asin | Token::Acos | Token::Atan | Token::Sinh | Token::Cosh | Token::Tanh | Token::Asinh | Token::Acosh | Token::Atanh | Token::Fact | Token::Floor | Token::Ceil | Token::Round | Token::Trunc | Token::Int | Token::Fract | Token::Cbrt | Token::Exp | Token::Re | Token::Im | Token::Conj | Token::Arg | Token::Mean | Token::Median | Token::VarFunc | Token::Std | Token::Det | Token::Inv | Token::Transpose | Token::Sort | Token::Tr => {
                if stack.is_empty() {
                    return Err(CalculatorError::InvalidExpression);
                }
                let arg = stack.pop().unwrap();
                let result = match token {
                    Token::Log => apply_function(&Function::Log, arg)?,
                    Token::Ln => apply_function(&Function::Ln, arg)?,
                    Token::Sin => apply_function(&Function::Sin, arg)?,
                    Token::Cos => apply_function(&Function::Cos, arg)?,
                    Token::Tan => apply_function(&Function::Tan, arg)?,
                    Token::Ctan => apply_function(&Function::Ctan, arg)?,
                    Token::LogBase(base) => {
                        let val = arg.to_scalar()?;
                        Value::Scalar(val.log(*base))
                    }
                    Token::Sqrt => apply_function(&Function::Sqrt, arg)?,
                    Token::Abs => apply_function(&Function::Abs, arg)?,
                    Token::Asin => apply_function(&Function::Asin, arg)?,
                    Token::Acos => apply_function(&Function::Acos, arg)?,
                    Token::Atan => apply_function(&Function::Atan, arg)?,
                    Token::Sinh => apply_function(&Function::Sinh, arg)?,
                    Token::Cosh => apply_function(&Function::Cosh, arg)?,
                    Token::Tanh => apply_function(&Function::Tanh, arg)?,
                    Token::Asinh => apply_function(&Function::Asinh, arg)?,
                    Token::Acosh => apply_function(&Function::Acosh, arg)?,
                    Token::Atanh => apply_function(&Function::Atanh, arg)?,
                    Token::Fact => apply_function(&Function::Fact, arg)?,
                    Token::Floor => apply_function(&Function::Floor, arg)?,
                    Token::Ceil => apply_function(&Function::Ceil, arg)?,
                    Token::Round => apply_function(&Function::Round, arg)?,
                    Token::Trunc => apply_function(&Function::Trunc, arg)?,
                    Token::Int => apply_function(&Function::Int, arg)?,
                    Token::Fract => apply_function(&Function::Fract, arg)?,
                    Token::Cbrt => apply_function(&Function::Cbrt, arg)?,
                    Token::Exp => apply_function(&Function::Exp, arg)?,
                    Token::Re => apply_function(&Function::Re, arg)?,
                    Token::Im => apply_function(&Function::Im, arg)?,
                    Token::Conj => apply_function(&Function::Conj, arg)?,
                    Token::Arg => apply_function(&Function::Arg, arg)?,
                    Token::Mean => apply_function(&Function::Mean, arg)?,
                    Token::Median => apply_function(&Function::Median, arg)?,
                    Token::VarFunc => apply_function(&Function::Var, arg)?,
                    Token::Std => apply_function(&Function::Std, arg)?,
                    Token::Det => apply_function(&Function::Det, arg)?,
                    Token::Inv => apply_function(&Function::Inv, arg)?,
                    Token::Transpose => apply_function(&Function::Transpose, arg)?,
                    Token::Sort => apply_function(&Function::Sort, arg)?,
                    Token::Tr => apply_function(&Function::Tr, arg)?,
                    _ => unreachable!(),
                };
                stack.push(result);
            },
            Token::Min | Token::Max | Token::Gcd | Token::Lcm | Token::Ncr | Token::Npr | Token::Root | Token::Polar | Token::Cov | Token::Corr | Token::LinReg => {
                if stack.len() < 2 {
                    return Err(CalculatorError::InvalidExpression);
                }
                let rhs = stack.pop().unwrap();
                let lhs = stack.pop().unwrap();
                let result = match token {
                    Token::Min => apply_function2(&Function2::Min, lhs, rhs)?,
                    Token::Max => apply_function2(&Function2::Max, lhs, rhs)?,
                    Token::Gcd => apply_function2(&Function2::Gcd, lhs, rhs)?,
                    Token::Lcm => apply_function2(&Function2::Lcm, lhs, rhs)?,
                    Token::Ncr => apply_function2(&Function2::Ncr, lhs, rhs)?,
                    Token::Npr => apply_function2(&Function2::Npr, lhs, rhs)?,
                    Token::Root => apply_function2(&Function2::Root, lhs, rhs)?,
                    Token::Polar => apply_function2(&Function2::Polar, lhs, rhs)?,
                    Token::Cov => apply_function2(&Function2::Cov, lhs, rhs)?,
                    Token::Corr => apply_function2(&Function2::Corr, lhs, rhs)?,
                    Token::LinReg => apply_function2(&Function2::LinReg, lhs, rhs)?,
                    _ => unreachable!(),
                };
                stack.push(result);
            },
            _ => return Err(CalculatorError::UnexpectedToken),
        }
    }

    if stack.len() != 1 {
        return Err(CalculatorError::InvalidExpression);
    }

    stack.pop().ok_or(CalculatorError::InvalidExpression)
}
