# Calculadora Rust (Rust + C# / WPF)

![C#](https://img.shields.io/badge/C%23-239120?style=for-the-badge&logo=c-sharp&logoColor=white)
![.NET](https://img.shields.io/badge/.NET-5C2D91?style=for-the-badge&logo=.net&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![WPF](https://img.shields.io/badge/WPF-blue?style=for-the-badge&logo=windows&logoColor=white)

**Calculadora Rust** es una aplicación científica e interactiva de alto rendimiento. Combina una interfaz gráfica de usuario moderna desarrollada en **C# con WPF (Windows Presentation Foundation)** bajo el patrón de arquitectura **MVVM**, y un motor matemático ultrarrápido desarrollado en **Rust** (compilado como una biblioteca dinámica nativa `.dll`).

Esta integración híbrida permite resolver operaciones matemáticas complejas, desde aritmética básica hasta álgebra lineal con matrices, estadística descriptiva, regresiones, cálculo numérico diferencial e integral, probabilidades y graficación en tiempo real, garantizando precisión numérica absoluta y latencias de microsegundos.

---

## 🏛️ Arquitectura del Sistema

La comunicación entre la interfaz de usuario en C# y el motor nativo en Rust se gestiona mediante interoperabilidad directa empleando **P/Invoke (Platform Invoke)** y **FFI (Foreign Function Interface)**:

```mermaid
graph TD
    A["Interfaz WPF (CalculatorView.xaml)"] -->|Data Binding / MVVM| B["Controlador de Estado (CalculatorViewModel.cs)"]
    B -->|P/Invoke - UTF-8 Strings| C["Puente FFI (CalculatorBridge.cs)"]
    C -->|Llamadas nativas DLL| D["Biblioteca Dinámica (calculator_core.dll)"]
    
    subgraph "Motor de Cálculo en Rust (Core)"
        D --> E["Analizador Léxico (lexer.rs)"]
        E -->|Preprocesamiento de Tokens| F["Multiplicación Implícita e Infix Auto-close"]
        F -->|Tokens Validados| G["Analizador Sintáctico (parser.rs)"]
        G -->|AST (Árbol de Sintaxis)| H["Evaluador Matemático (evaluator.rs)"]
        H -->|Memoria y ANS| I["Estado Mutable (value.rs)"]
    end
```

### Flujo de Ejecución del Cálculo

1. **Entrada de Datos (C#)**: El usuario escribe la expresión o hace clic en los botones de la interfaz. La expresión se enlaza dinámicamente al `CalculatorViewModel`.
2. **Transferencia de Memoria**: La cadena se envía a través de [CalculatorBridge.cs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/CalculatorGui/CalculatorBridge.cs) usando punteros nativos (`IntPtr`) codificados en UTF-8. Rust recibe la dirección de memoria, garantizando una transferencia sin copias costosas.
3. **Procesamiento de Tokens (Rust)**:
   - **Lexer**: Divide la entrada en componentes atómicos (números, operadores, funciones). Inserta dinámicamente operadores implícitos (por ejemplo, convierte `2pi(x)` en `2 * pi * (x)`) y balancea de forma automática los paréntesis abiertos pendientes al final de la expresión.
   - **Parser**: Valida la gramática matemática y construye el **Árbol de Sintaxis Abstracta (AST)** respetando las reglas de precedencia estándar.
   - **Evaluator**: Evalúa el AST de manera recursiva mediante tipos algebraicos seguros (`Value`). Almacena las variables definidas y actualiza la memoria especial `ans`.
4. **Liberación de Memoria**: C# solicita a Rust que libere el búfer nativo que contiene el resultado mediante `free_string_ffi` para evitar fugas de memoria.

---

## 🎨 Visor Natural Bidimensional y Diseño Físico (WPF)

La interfaz de usuario ha sido rediseñada para ofrecer una experiencia prémium similar a la de una calculadora científica física:

### 1. Visor Matemático Natural (2D Natural Display)

Implementado mediante un control personalizado avanzado ([MathDisplayControl.cs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/CalculatorGui/View/MathDisplayControl.cs)):

- **Visualización Bidimensional**: Renderiza automáticamente fórmulas matemáticas complejas como si estuvieran en un libro de texto:
  - **Fracciones**: Divide el numerador y denominador con una línea fraccionaria horizontal de tamaño auto-ajustable.
  - **Potencias**: Renderiza exponentes en un tamaño reducido (`fontSize * 0.65`) situándolos en la esquina superior derecha de la base.
  - **Raíces cuadradas**: Dibuja el radical `√` junto con una barra superior continua (`Border`) que se estira y cubre exactamente todo el radicando.
  - **Paréntesis**: Aplica paréntesis envolventes escalados verticalmente al tamaño de su contenido.
  - **Sustitución de caracteres**: Reemplaza instantáneamente los operadores del motor por notación limpia tradicional (`*` $\rightarrow$ `×`, `pi` $\rightarrow$ `π`, `theta` $\rightarrow$ `θ`, `sqrt` $\rightarrow$ `√`).
- **Edición Fluida (Foco del Editor)**: Cuando el usuario hace clic para editar, el visor natural se oculta dando paso a un cuadro de texto convencional (`TextBox`) para permitir una edición ágil. Al perder el foco o presionar `=` / `Enter`, el visor natural recompone y muestra la expresión con su tipografía y renderizado bidimensional.
- **Tolerancia a Errores (Fallback)**: Si el usuario escribe una expresión inválida a mitad de edición, el control muestra la entrada de texto limpio de manera robusta sin interrumpir la escritura ni lanzar errores visuales.

### 2. Estructura de Mano (Handheld Design)

- **Diseño Vertical**: Dimensionado con un ancho vertical de `380px` y alto de `600px`, emulando el factor de forma físico de las calculadoras científicas tradicionales de mano.

- **Botonera Ovalada**: Los botones usan la tipografía y colores corporativos con un `CornerRadius="12"`, lo que les da una apariencia ergonómica tridimensional.
- **Menús Rápidos de Cuadrícula (UniformGrid Popups)**: Reemplazo de los menús contextuales planos por elementos emergentes `Popup` reactivos. Los paneles de **Trigonometría** y **Funciones** muestran rejillas organizadas de botones de acceso directo (e.g., `sin`, `cos`, `tan`, `asin`, `acos`, `atan` con sus equivalentes hiperbólicos) organizadas en cuadrículas que se cierran automáticamente tras insertar la función.

---

## 🚀 Módulos Avanzados del Motor (Rust)

El motor de Rust cuenta con soporte nativo optimizado para una amplia variedad de disciplinas matemáticas:

### 1. Álgebra Lineal y Matrices

Soporta vectores y matrices usando la sintaxis de Matlab: `[valores_fila1; valores_fila2]`.

- **Ejemplo**: `[1, 2; 3, 4]` define una matriz de 2x2.
- **Operaciones Soportadas**:
  - **Determinante (`det`)**: `det([1, 2; 3, 4])` $\rightarrow$ `-2`.
  - **Inversa (`inv`)**: `inv([1, 2; 3, 4])` $\rightarrow$ `[-2, 1; 1.5, -0.5]`.
  - **Transpuesta (`transpose` / `trans`)**: `transpose([1, 2])` $\rightarrow$ `[1; 2]`.

### 2. Números Complejos

Soporte nativo integrado en el sistema dinámico de datos (`Value::Complex`) con parte real e imaginaria.

- **Sintaxis**: `3 + 4i` o `1 - i`.
- **Operaciones Soportadas**:
  - **Componentes (`re`, `im`)**: `re(3 + 4i)` $\rightarrow$ `3`, `im(3 + 4i)` $\rightarrow$ `4`.
  - **Conjugado (`conj`)**: `conj(3 + 4i)` $\rightarrow$ `3 - 4i`.
  - **Fase (`arg`)**: Devuelve el argumento en radianes. `arg(i)` $\rightarrow$ `1.57079633`.
  - **Polar (`polar`)**: Construye un complejo a partir del módulo y fase: `polar(5, 0)`.

### 3. Cálculo y Análisis Numérico

El motor incluye solucionadores analítico-numéricos integrados directamente en el parser:

- **Derivada Numérica (`deriv`)**: Calcula la derivada de una función en un punto dado usando diferencias finitas.
  - *Sintaxis*: `deriv(expresión, variable, punto)` o `deriv(expresión, punto)`
  - *Ejemplo*: `deriv(x^2, x, 3)` $\rightarrow$ `6`
- **Integral Definida (`intg`)**: Calcula el área bajo la curva mediante el método adaptativo de Simpson.
  - *Sintaxis*: `intg(expresión, variable, inferior, superior)`
  - *Ejemplo*: `intg(x^2, x, 0, 3)` $\rightarrow$ `9`
- **Sumatoria (`sum`) y Productoria (`prod`)**: Bucles de rango cerrado.
  - *Ejemplo*: `sum(i^2, i, 1, 4)` $\rightarrow$ `30` (calcula $1^2 + 2^2 + 3^2 + 4^2$).
  - *Ejemplo*: `prod(i, i, 1, 5)` $\rightarrow$ `120` (calcula el factorial de 5).

### 4. Estadística Descriptiva y Regresiones

- **Estadísticas de Vectores**: `mean([1,2,3,4])` (media), `median([1,2,3,4])` (mediana), `var([1,2,3,4])` (varianza), `std([1,2,3,4])` (desviación estándar).

- **Covarianza y Correlación**: `cov(vec1, vec2)` y `corr(vec1, vec2)`.
- **Regresión Lineal (`linreg`)**: Calcula la línea de mejor ajuste por mínimos cuadrados.
  - *Sintaxis*: `linreg([x1, x2, ...], [y1, y2, ...])` $\rightarrow$ `[pendiente, intercepto, correlación_r, determinación_R²]`.
- **Regresión Polinómica (`polyreg`)**: Ajusta un polinomio de grado $n$ a los puntos dados.
  - *Sintaxis*: `polyreg(x_vector, y_vector, grado)`.

### 5. Probabilidades y Distribuciones

- **Distribución Normal**: Densidad `normpdf(x, media, desv)` y acumulada `normcdf(x, media, desv)`.

- **Distribución Binomial**: `binopdf(k, n, p)` y `binocdf(k, n, p)`.
- **Distribución de Poisson**: `poisspdf(k, lambda)` y `poisscdf(k, lambda)`.
- **Generación Aleatoria (`rand`)**: `rand(min, max)` devuelve un número pseudoaleatorio en el rango.

### 6. Sistema de Asignación y Memoria

- **Declaración de Variables**: Permite asignar resultados a variables en la memoria utilizando `=` (ej. `x = 5`, `Radio = 12`).

- **Variable del Último Cálculo (`ans`)**: Conserva el último resultado válido obtenido de forma automática.
- **RPN Bypass**: Si el motor de Rust detecta operandos al estilo RPN (Notación Polaca Inversa), activa automáticamente la pila nativa y descarta el preprocesamiento infijo.

---

## 📂 Estructura del Código Fuente

Los archivos clave del proyecto se organizan de la siguiente manera:

- **[src/lib.rs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/src/lib.rs)**: Define los puntos de entrada exportados de la DLL (`extern "C"`), las funciones FFI de cálculo, conversión de unidades físicas y los bloques `catch_unwind` para aislar pánicos de Rust.
- **[src/calculator/ast.rs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/src/calculator/ast.rs)**: Representa los nodos jerárquicos del compilador de expresiones (números, operadores, matrices, etc.).
- **[src/calculator/token.rs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/src/calculator/token.rs)**: Estructuras del diccionario léxico de la calculadora.
- **[src/calculator/lexer.rs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/src/calculator/lexer.rs)**: Analizador léxico y motor de re-tokenización (multiplicación implícita, auto-cierre de paréntesis).
- **[src/calculator/parser.rs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/src/calculator/parser.rs)**: Traduce la secuencia de tokens en el árbol AST empleando el algoritmo Pratt Parser para precedencia de operadores.
- **[src/calculator/evaluator.rs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/src/calculator/evaluator.rs)**: Evaluación de expresiones a partir de las ramas del AST, funciones de matrices, estadística y RPN.
- **[src/calculator/value.rs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/src/calculator/value.rs)**: Representación de tipos y almacenamiento mutable en Rust (números reales, complejos, matrices, booleanos y errores estructurados).
- **[CalculatorGui/CalculatorBridge.cs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/CalculatorGui/CalculatorBridge.cs)**: Puente C# que maneja los comandos de P/Invoke y conversión segura de cadenas ANSI/UTF-8.
- **[CalculatorGui/View/MathDisplayControl.cs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/CalculatorGui/View/MathDisplayControl.cs)**: Lógica del parser de visualización matemática bidimensional nativa para WPF.
- **[CalculatorGui/View/CalculatorView.xaml.cs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/CalculatorGui/View/CalculatorView.xaml.cs)**: Controla los eventos de la UI, los popups de Trigonometría/Funciones y la alternancia dinámica del foco LCD.
- **[CalculatorGui/ViewModel/CalculatorViewModel.cs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/CalculatorGui/ViewModel/CalculatorViewModel.cs)**: Lógica MVVM que gestiona la entrada, historial y sincronización de hilos con la biblioteca en Rust.

---

## 🧪 Suite de Pruebas y Robustez

El motor matemático se valida mediante una extensa y robusta suite que cuenta con **539 pruebas automatizadas** diseñadas para verificar la precisión del sistema bajo múltiples condiciones límite:

### Categorías de Pruebas

1. **Pruebas Estáticas (≈350 pruebas en [suite_static.rs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/src/calculator/suite_static.rs))**:
   - Comprueban operaciones trigonométricas en grados, radianes y gradianes.
   - Verifican constantes matemáticas (`pi`, `e`) y operaciones aritméticas estándar.
2. **Pruebas de Regresión de Bugs (≈20 pruebas en [suite_regression.rs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/src/calculator/suite_regression.rs))**:
   - Re-evalúan fallos detectados con anterioridad para evitar su reaparición (ej. pánicos con divisiones entre cero complejas o matrices sin inicializar).
3. **Pruebas de Estrés y Límites Numéricos (≈160 pruebas en [suite_stress.rs](file:///C:/Users/hsm76/Documents/GitHub/Calculadora-Rust/src/calculator/suite_stress.rs))**:
   - **Límites de Recursión**: Evalúa el parser ante expresiones con anidamiento profundo de paréntesis (comprobando que el límite de profundidad a `500` previene pánicos por desbordamiento de pila en Rust).
   - **Robustez del IEEE 754**: Evalúa precisión ante valores infinitesimales (subnormales), infinitos (`inf`), cancelaciones catastróficas y divisiones por cero indeterminadas.
   - **Evaluaciones en Cadena (`ans`)**: Valida acumulaciones numéricas mediante la variable `ans` a lo largo de bucles encadenados de más de 10 iteraciones.

Para ejecutar todas las pruebas del motor de cálculo:

```bash
cargo test
```

---

## 🛠️ Guía de Compilación y Configuración

### Prerrequisitos

1. [.NET SDK 8.0 / 10.0](https://dotnet.microsoft.com/download).
2. [Compilador de Rust y Cargo](https://www.rust-lang.org/tools/install) (versión estable).

### Construcción

1. **Compilar el Motor Nativo (Rust)**:
   En la raíz del repositorio, ejecuta:

   ```bash
   cargo build --release
   ```

   Esto compilará el código de Rust y generará el archivo `calculator_core.dll` en la carpeta `target/release/`.

2. **Ejecutar la Interfaz de Usuario (WPF)**:
   El archivo de proyecto `CalculatorGui.csproj` está configurado para copiar automáticamente la DLL de Rust generada a la carpeta de salida del binario de C#. Para iniciar la calculadora, ejecuta:

   ```bash
   dotnet run --project CalculatorGui/CalculatorGui.csproj
   ```
