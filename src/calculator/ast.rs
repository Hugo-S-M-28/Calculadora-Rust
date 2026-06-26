/// Representa el Árbol de Sintaxis Abstracta (AST) de una expresión matemática.
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum AST {
    /// Un número literal de punto flotante (ej. 3.14)
    Num(f64),
    /// Una variable por su nombre (ej. "x", "y")
    Var(String),
    /// Una operación binaria con una expresión izquierda, un operador y una expresión derecha
    BinOp(Box<AST>, Operator, Box<AST>),
    /// Una llamada a una función científica con un solo argumento (ej. sin(x))
    Func(Function, Box<AST>),
    /// Una llamada a una función científica con dos argumentos (ej. min(x, y))
    Func2(Function2, Box<AST>, Box<AST>),
    /// Una constante predefinida (ej. Pi, E, Tau, Phi)
    Const(f64),
    /// Logaritmo en base específica (base, expresión)
    LogBase(f64, Box<AST>),
    /// Derivada numérica (expresión, variable_name, punto_evaluacion)
    Deriv(Box<AST>, String, Box<AST>),
    /// Integral definida (expresión, variable_name, límite_inferior, límite_superior)
    Intg(Box<AST>, String, Box<AST>, Box<AST>),
    /// Sumatorio (expresión, variable_name, inicio, fin)
    Sum(Box<AST>, String, Box<AST>, Box<AST>),
    /// Productorio (expresión, variable_name, inicio, fin)
    Prod(Box<AST>, String, Box<AST>, Box<AST>),
    /// Representación de un literal matricial/vectorial (filas de columnas de expresiones)
    MatrixLiteral(Vec<Vec<AST>>),
    /// Regresión polinómica (x, y, grado)
    PolyReg(Box<AST>, Box<AST>, Box<AST>),
    /// Función probabilística o aleatoria (función, argumentos)
    ProbFunc(ProbFunction, Vec<AST>),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ProbFunction {
    Rand,
    NormPdf,
    NormCdf,
    BinoPdf,
    BinoCdf,
    PoissPdf,
    PoissCdf,
}

/// Operadores aritméticos binarios soportados por la calculadora.
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Operator {
    /// Suma (+)
    Add,
    /// Resta (-)
    Sub,
    /// Multiplicación (*)
    Mul,
    /// División (/)
    Div,
    /// Módulo o resto de división (%)
    Mod,
    /// Porcentaje (%)
    Percent,
    /// Potencia (^)
    Power,
}

/// Funciones matemáticas científicas de un solo argumento.
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Function {
    /// Logaritmo en base 10 (log)
    Log,
    /// Logaritmo natural (ln)
    Ln,
    /// Seno (sin)
    Sin,
    /// Coseno (cos)
    Cos,
    /// Tangente (tan)
    Tan,
    /// Cotangente (ctan)
    Ctan,
    /// Raíz cuadrada (sqrt)
    Sqrt,
    /// Valor absoluto (abs)
    Abs,
    /// Arcoseno (asin)
    Asin,
    /// Arcocoseno (acos)
    Acos,
    /// Arcotangente (atan)
    Atan,
    /// Seno hiperbólico (sinh)
    Sinh,
    /// Coseno hiperbólico (cosh)
    Cosh,
    /// Tangente hiperbólica (tanh)
    Tanh,
    /// Arcoseno hiperbólico (asinh)
    Asinh,
    /// Arcocoseno hiperbólico (acosh)
    Acosh,
    /// Arcotangente hiperbólica (atanh)
    Atanh,
    /// Factorial (!)
    Fact,
    /// Función piso (floor)
    Floor,
    /// Función techo (ceil)
    Ceil,
    /// Redondeo al entero más cercano (round)
    Round,
    /// Truncamiento (trunc)
    Trunc,
    /// Parte entera (int)
    Int,
    /// Parte fraccionaria (fract)
    Fract,
    /// Raíz cúbica (cbrt)
    Cbrt,
    /// Parte real (re)
    Re,
    /// Parte imaginaria (im)
    Im,
    /// Conjugado complejo (conj)
    Conj,
    /// Argumento complejo (arg)
    Arg,
    /// Función exponencial (exp)
    Exp,
    /// Media aritmética (mean)
    Mean,
    /// Mediana (median)
    Median,
    /// Varianza (var)
    Var,
    /// Desviación estándar (std)
    Std,
    /// Determinante de matriz (det)
    Det,
    /// Inversa de matriz (inv)
    Inv,
    /// Transpuesta de matriz (transpose)
    Transpose,
    /// Suma de elementos de un vector (sum)
    Sum,
    /// Ordenar vector ascendente (sort)
    Sort,
    /// Traza de matriz cuadrada (tr)
    Tr,
    /// Mínimo elemento de un vector (min_vec, complementa a Function2::Min)
    MinVec,
    /// Máximo elemento de un vector (max_vec, complementa a Function2::Max)
    MaxVec,
}

/// Funciones matemáticas científicas de dos argumentos.
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Function2 {
    /// Mínimo entre dos números (min)
    Min,
    /// Máximo entre dos números (max)
    Max,
    /// Residuo de división o módulo (mod)
    Mod,
    /// Máximo común divisor (gcd)
    Gcd,
    /// Mínimo común múltiplo (lcm)
    Lcm,
    /// Combinación nCr (ncr)
    Ncr,
    /// Permutación nPr (npr)
    Npr,
    /// Raíz enésima (root)
    Root,
    /// Forma polar (polar)
    Polar,
    /// Covarianza (cov)
    Cov,
    /// Correlación (corr)
    Corr,
    /// Regresión lineal (linreg)
    LinReg,
    /// Logaritmo en base arbitraria (log)
    LogBase,
}