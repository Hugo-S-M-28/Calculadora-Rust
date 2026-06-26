/// Representa los componentes léxicos (tokens) en los que se divide una expresión matemática.
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Token {
    /// Un valor numérico de tipo flotante
    Number(f64),
    /// Operador de suma (+)
    Plus,
    /// Operador de resta (-)
    Minus,
    /// Operador de multiplicación (*)
    Multiply,
    /// Operador de división (/)
    Divide,
    /// Módulo matemático: usado internamente por la función `mod(a, b)`.
    /// Semántica: resto de la división entera con el mismo signo que el divisor.
    /// NOTA: El carácter `%` en el lexer se tokeniza como `Token::Percent` (porcentaje),
    /// no como `Token::Mod`. Este token solo aparece por la ruta de la función `mod()`.
    Mod,
    /// Operador de porcentaje `%`: en contexto binario, `a % b` equivale a `a * b / 100`.
    /// Por ejemplo: `200 % 15` = `200 * 15 / 100` = `30`.
    /// No es lo mismo que el módulo matemático — ver `Token::Mod`.
    Percent,
    /// Paréntesis izquierdo (
    LeftParenthesis,
    /// Paréntesis derecho )
    RightParenthesis,
    /// Coma (,) para separar argumentos en funciones de múltiples variables
    Comma,
    /// Variable con su identificador (ej. "x")
    Variable(String),
    /// Signo de igualdad (=) utilizado en resolución de ecuaciones
    Equal,
    /// Función logaritmo base 10 (log)
    Log,
    /// Función logaritmo natural (ln)
    Ln,
    /// Función seno (sin)
    Sin,
    /// Función coseno (cos)
    Cos,
    /// Función tangente (tan)
    Tan,
    /// Función cotangente (ctan)
    Ctan,
    /// Constante Pi (3.14159...)
    Pi,
    /// Constante de Euler e (2.71828...)
    E,
    /// Constante Tau (2 * Pi)
    Tau,
    /// Constante Phi (Proporción áurea ~1.618)
    Phi,
    /// Constante raíz cuadrada de 2
    Sqrt2,
    /// Logaritmo en una base específica personalizada, ej. log2
    LogBase(f64),
    /// Operador de potencia (^)
    Power,
    /// Raíz cuadrada (sqrt)
    Sqrt,
    /// Valor absoluto (abs)
    Abs,
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
    /// Raíz enésima (root)
    Root,
    /// Parte real de un complejo (re)
    Re,
    /// Parte imaginaria de un complejo (im)
    Im,
    /// Conjugado de un complejo (conj)
    Conj,
    /// Argumento de un complejo (arg)
    Arg,
    /// Crear complejo en forma polar (polar)
    Polar,
    /// Operador factorial posfijo (!)
    Excl,
    /// Constante de velocidad de la luz c (299792458 m/s)
    ConstC,
    /// Constante de Planck h (6.62607015e-34 J s)
    ConstH,
    /// Constante de gravitación universal G (6.6743e-11 m^3 kg^-1 s^-2)
    ConstG,
    /// Función exponencial e^x (exp)
    Exp,
    /// Mínimo entre dos números (min)
    Min,
    /// Máximo entre dos números (max)
    Max,
    /// Máximo común divisor (gcd)
    Gcd,
    /// Mínimo común múltiplo (lcm)
    Lcm,
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
    /// Combinación nCr
    Ncr,
    /// Permutación nPr
    Npr,
    /// Derivada numérica (deriv)
    Deriv,
    /// Integral definida (intg)
    Intg,
    /// Sumatoria (sum)
    Sum,
    /// Productoria (prod)
    Prod,
    /// Corchete izquierdo ([)
    LeftBracket,
    /// Corchete derecho (])
    RightBracket,
    /// Punto y coma (;)
    Semicolon,
    /// Media aritmética (mean)
    Mean,
    /// Mediana (median)
    Median,
    /// Varianza (var)
    VarFunc,
    /// Desviación estándar (std)
    Std,
    /// Covarianza (cov)
    Cov,
    /// Correlación (corr)
    Corr,
    /// Regresión lineal (linreg)
    LinReg,
    /// Regresión polinómica (polyreg)
    PolyReg,
    /// Determinante de matriz (det)
    Det,
    /// Inversa de matriz (inv)
    Inv,
    /// Transpuesta de matriz (transpose o trans)
    Transpose,
    /// Generación de números aleatorios (rand)
    Rand,
    /// Densidad de probabilidad normal (normpdf)
    NormPdf,
    /// Distribución acumulada normal (normcdf)
    NormCdf,
    /// Densidad de probabilidad binomial (binopdf)
    BinoPdf,
    /// Distribución acumulada binomial (binocdf)
    BinoCdf,
    /// Densidad de probabilidad Poisson (poisspdf)
    PoissPdf,
    /// Distribución acumulada Poisson (poisscdf)
    PoissCdf,
    /// Ordena un vector ascendente (sort)
    Sort,
    /// Traza de una matriz cuadrada (tr)
    Tr,
    /// Representa el último resultado calculado (ANS)
    Ans,
}
