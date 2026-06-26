/// Recorrido del AST para sustituir una variable por un valor numérico concreto.
/// Usado por las funciones de graficación FFI (evaluate_with_var, parametric, polar).

use crate::calculator::ast::AST;

/// Sustituye recursivamente todas las ocurrencias de `var_name` en el AST
/// por el nodo `AST::Num(val)`.
pub fn substitute_variable(ast: &AST, var_name: &str, val: f64) -> AST {
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
            let new_rows = rows.iter().map(|row| {
                row.iter().map(|element| substitute_variable(element, var_name, val)).collect()
            }).collect();
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
