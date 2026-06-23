use num_complex::Complex;
use nalgebra::DMatrix;
use std::fmt;
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;
use crate::calculator::calculator::CalculatorError;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Scalar(f64),
    Complex(Complex<f64>),
    Vector(Vec<f64>),
    Matrix(DMatrix<f64>),
}

// Variables map storage
pub fn get_variables() -> &'static Mutex<HashMap<String, Value>> {
    static VARIABLES: OnceLock<Mutex<HashMap<String, Value>>> = OnceLock::new();
    VARIABLES.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert("i".to_string(), Value::Complex(Complex::new(0.0, 1.0)));
        map.insert("j".to_string(), Value::Complex(Complex::new(0.0, 1.0)));
        Mutex::new(map)
    })
}

pub fn get_variable(name: &str) -> Option<Value> {
    get_variables().lock().ok().and_then(|g| g.get(name).cloned())
}

pub fn set_variable(name: &str, val: Value) {
    if let Ok(mut g) = get_variables().lock() {
        g.insert(name.to_string(), val);
    }
}

#[allow(dead_code)]
pub fn clear_variables() {
    if let Ok(mut g) = get_variables().lock() {
        g.clear();
    }
}

impl Value {
    pub fn to_scalar(&self) -> Result<f64, CalculatorError> {
        match self {
            Value::Scalar(n) => Ok(*n),
            Value::Complex(c) if c.im == 0.0 => Ok(c.re),
            _ => Err(CalculatorError::InvalidExpression),
        }
    }

    pub fn add(self, other: Value) -> Result<Value, CalculatorError> {
        match (self, other) {
            (Value::Scalar(a), Value::Scalar(b)) => Ok(Value::Scalar(a + b)),
            (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a + b)),
            (Value::Scalar(a), Value::Complex(b)) => Ok(Value::Complex(Complex::new(a, 0.0) + b)),
            (Value::Complex(a), Value::Scalar(b)) => Ok(Value::Complex(a + Complex::new(b, 0.0))),
            (Value::Vector(mut a), Value::Vector(b)) => {
                if a.len() != b.len() {
                    return Err(CalculatorError::InvalidExpression);
                }
                for i in 0..a.len() {
                    a[i] += b[i];
                }
                Ok(Value::Vector(a))
            }
            (Value::Matrix(a), Value::Matrix(b)) => {
                if a.nrows() != b.nrows() || a.ncols() != b.ncols() {
                    return Err(CalculatorError::InvalidExpression);
                }
                Ok(Value::Matrix(a + b))
            }
            _ => Err(CalculatorError::InvalidExpression),
        }
    }
    
    pub fn sub(self, other: Value) -> Result<Value, CalculatorError> {
        match (self, other) {
            (Value::Scalar(a), Value::Scalar(b)) => Ok(Value::Scalar(a - b)),
            (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a - b)),
            (Value::Scalar(a), Value::Complex(b)) => Ok(Value::Complex(Complex::new(a, 0.0) - b)),
            (Value::Complex(a), Value::Scalar(b)) => Ok(Value::Complex(a - Complex::new(b, 0.0))),
            (Value::Vector(mut a), Value::Vector(b)) => {
                if a.len() != b.len() {
                    return Err(CalculatorError::InvalidExpression);
                }
                for i in 0..a.len() {
                    a[i] -= b[i];
                }
                Ok(Value::Vector(a))
            }
            (Value::Matrix(a), Value::Matrix(b)) => {
                if a.nrows() != b.nrows() || a.ncols() != b.ncols() {
                    return Err(CalculatorError::InvalidExpression);
                }
                Ok(Value::Matrix(a - b))
            }
            _ => Err(CalculatorError::InvalidExpression),
        }
    }

    pub fn mul(self, other: Value) -> Result<Value, CalculatorError> {
        match (self, other) {
            (Value::Scalar(a), Value::Scalar(b)) => Ok(Value::Scalar(a * b)),
            (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a * b)),
            (Value::Scalar(a), Value::Complex(b)) => Ok(Value::Complex(Complex::new(a, 0.0) * b)),
            (Value::Complex(a), Value::Scalar(b)) => Ok(Value::Complex(a * Complex::new(b, 0.0))),
            (Value::Scalar(a), Value::Vector(mut v)) => {
                for x in &mut v {
                    *x *= a;
                }
                Ok(Value::Vector(v))
            }
            (Value::Vector(mut v), Value::Scalar(a)) => {
                for x in &mut v {
                    *x *= a;
                }
                Ok(Value::Vector(v))
            }
            (Value::Scalar(a), Value::Matrix(m)) => Ok(Value::Matrix(m * a)),
            (Value::Matrix(m), Value::Scalar(a)) => Ok(Value::Matrix(m * a)),
            (Value::Matrix(m), Value::Vector(v)) => {
                if m.ncols() != v.len() {
                    return Err(CalculatorError::InvalidExpression);
                }
                let n_vec = nalgebra::DVector::from_vec(v);
                let res = m * n_vec;
                Ok(Value::Vector(res.as_slice().to_vec()))
            }
            (Value::Matrix(a), Value::Matrix(b)) => {
                if a.ncols() != b.nrows() {
                    return Err(CalculatorError::InvalidExpression);
                }
                Ok(Value::Matrix(a * b))
            }
            _ => Err(CalculatorError::InvalidExpression),
        }
    }

    pub fn div(self, other: Value) -> Result<Value, CalculatorError> {
        match (self, other) {
            (Value::Scalar(a), Value::Scalar(b)) => {
                if b == 0.0 {
                    Err(CalculatorError::DivisionByZero)
                } else {
                    Ok(Value::Scalar(a / b))
                }
            }
            (Value::Complex(a), Value::Complex(b)) => {
                if b.norm() == 0.0 {
                    Err(CalculatorError::DivisionByZero)
                } else {
                    Ok(Value::Complex(a / b))
                }
            }
            (Value::Complex(a), Value::Scalar(b)) => {
                if b == 0.0 {
                    Err(CalculatorError::DivisionByZero)
                } else {
                    Ok(Value::Complex(a / b))
                }
            }
            (Value::Scalar(a), Value::Complex(b)) => {
                if b.norm() == 0.0 {
                    Err(CalculatorError::DivisionByZero)
                } else {
                    Ok(Value::Complex(Complex::new(a, 0.0) / b))
                }
            }
            _ => Err(CalculatorError::InvalidExpression),
        }
    }

    pub fn pow(self, other: Value) -> Result<Value, CalculatorError> {
        match (self, other) {
            (Value::Scalar(a), Value::Scalar(b)) => Ok(Value::Scalar(a.powf(b))),
            (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a.powc(b))),
            (Value::Complex(a), Value::Scalar(b)) => Ok(Value::Complex(a.powf(b))),
            (Value::Scalar(a), Value::Complex(b)) => Ok(Value::Complex(Complex::new(a, 0.0).powc(b))),
            _ => Err(CalculatorError::InvalidExpression),
        }
    }

    pub fn rem(self, other: Value) -> Result<Value, CalculatorError> {
        match (self, other) {
            (Value::Scalar(a), Value::Scalar(b)) => {
                if b == 0.0 {
                    Err(CalculatorError::DivisionByZero)
                } else {
                    Ok(Value::Scalar(a - (a / b).floor() * b))
                }
            }
            _ => Err(CalculatorError::InvalidExpression),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let clean = |val: f64| {
            let r = (val * 100000000.0).round() / 100000000.0;
            if r == -0.0 { 0.0 } else { r }
        };
        match self {
            Value::Scalar(n) => {
                if n.is_nan() {
                    write!(f, "NaN")
                } else if n.is_infinite() {
                    if *n < 0.0 {
                        write!(f, "-Infinity")
                    } else {
                        write!(f, "Infinity")
                    }
                } else {
                    write!(f, "{}", clean(*n))
                }
            }
            Value::Complex(c) => {
                let r = clean(c.re);
                let i = clean(c.im);
                if i == 0.0 {
                    write!(f, "{}", r)
                } else if r == 0.0 {
                    write!(f, "{}i", i)
                } else if i < 0.0 {
                    write!(f, "{} - {}i", r, -i)
                } else {
                    write!(f, "{} + {}i", r, i)
                }
            }
            Value::Vector(v) => {
                write!(f, "[")?;
                for (idx, val) in v.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", clean(*val))?;
                }
                write!(f, "]")
            }
            Value::Matrix(m) => {
                write!(f, "[")?;
                for r in 0..m.nrows() {
                    if r > 0 {
                        write!(f, "; ")?;
                    }
                    for c in 0..m.ncols() {
                        let val = m[(r, c)];
                        if c > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", clean(val))?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}
