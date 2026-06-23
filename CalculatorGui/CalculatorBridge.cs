using System;
using System.Runtime.InteropServices;
using System.Text;

namespace CalculatorGui
{
    // Estructura para retornar resultados desde el motor en Rust.
    // Contiene un puntero a la cadena de texto de resultado y un indicador de éxito.
    [StructLayout(LayoutKind.Sequential)]
    public struct CalculatorResult
    {
        public IntPtr Value;
        public byte IsSuccess;
    }

    /// <summary>
    /// Puente que realiza las llamadas al motor matemático desarrollado en Rust.
    /// Utiliza P/Invoke (Platform Invoke) para interactuar con la biblioteca nativa "calculator_core.dll".
    /// </summary>
    public static class CalculatorBridge
    {
        // Importación de funciones externas de calculator_core.dll (FFI - Foreign Function Interface)
        [DllImport("calculator_core.dll", EntryPoint = "process_expression_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern CalculatorResult process_expression_ffi(IntPtr inputPtr);

        [DllImport("calculator_core.dll", EntryPoint = "process_expression_preview_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern CalculatorResult process_expression_preview_ffi(IntPtr inputPtr);

        [DllImport("calculator_core.dll", EntryPoint = "validate_expression_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern byte validate_expression_ffi(IntPtr inputPtr);

        [DllImport("calculator_core.dll", EntryPoint = "convert_units_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern double convert_units_ffi(double value, IntPtr fromUnitPtr, IntPtr toUnitPtr, IntPtr categoryPtr);

        [DllImport("calculator_core.dll", EntryPoint = "evaluate_with_var_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern double evaluate_with_var_ffi(IntPtr inputPtr, double varValue);

        [DllImport("calculator_core.dll", EntryPoint = "evaluate_parametric_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern double evaluate_parametric_ffi(IntPtr inputPtr, double tValue);

        [DllImport("calculator_core.dll", EntryPoint = "evaluate_polar_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern double evaluate_polar_ffi(IntPtr inputPtr, double thetaValue);

        [DllImport("calculator_core.dll", EntryPoint = "detect_notation_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern byte detect_notation_ffi(IntPtr inputPtr);

        [DllImport("calculator_core.dll", EntryPoint = "calculate_grid_step_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern double calculate_grid_step_ffi(double minVal, double maxVal);

        [DllImport("calculator_core.dll", EntryPoint = "free_string_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern void free_string_ffi(IntPtr ptr);

        [DllImport("calculator_core.dll", EntryPoint = "set_angle_mode_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern void set_angle_mode_ffi(byte mode);

        [DllImport("calculator_core.dll", EntryPoint = "get_last_result_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern double get_last_result_ffi();

        [DllImport("calculator_core.dll", EntryPoint = "clear_last_result_ffi", CallingConvention = CallingConvention.Cdecl)]
        private static extern void clear_last_result_ffi();

        /// <summary>
        /// Establece el modo de ángulo en el motor de Rust (false para radianes, true para grados).
        /// </summary>
        public static void SetAngleMode(bool degrees)
        {
            try
            {
                set_angle_mode_ffi(degrees ? (byte)1 : (byte)0);
            }
            catch { }
        }

        /// <summary>
        /// Obtiene el último resultado calculado guardado en la memoria de Rust (ANS).
        /// </summary>
        public static double GetLastResult()
        {
            try
            {
                return get_last_result_ffi();
            }
            catch
            {
                return 0.0;
            }
        }

        /// <summary>
        /// Limpia el último resultado (ANS) en la memoria de Rust.
        /// </summary>
        public static void ResetAns()
        {
            try
            {
                clear_last_result_ffi();
            }
            catch { }
        }

        /// <summary>
        /// Envía una expresión matemática al motor de Rust y retorna el resultado evaluado.
        /// Realiza la conversión manual de strings de .NET a UTF-8 nativo y libera la memoria de Rust de forma segura.
        /// </summary>
        public static (string Result, bool IsSuccess) ProcessExpression(string expression)
        {
            if (string.IsNullOrWhiteSpace(expression))
            {
                return ("Error: Expresión vacía", false);
            }

            IntPtr inputPtr = IntPtr.Zero;
            try
            {
                // Convierte la cadena C# a bytes codificados en UTF-8 y añade el carácter nulo terminador
                byte[] utf8Bytes = Encoding.UTF8.GetBytes(expression + "\0");
                inputPtr = Marshal.AllocHGlobal(utf8Bytes.Length);
                Marshal.Copy(utf8Bytes, 0, inputPtr, utf8Bytes.Length);

                // Llama a la función exportada de Rust
                CalculatorResult resStruct = process_expression_ffi(inputPtr);

                if (resStruct.Value == IntPtr.Zero)
                {
                    return ("Error: Resultado nulo", false);
                }

                // Convierte el puntero a cadena UTF-8 nativo a cadena de .NET
                string result = Marshal.PtrToStringUTF8(resStruct.Value) ?? "Error: Conversión de texto fallida";
                bool isSuccess = resStruct.IsSuccess != 0;

                // Libera la memoria de la cadena asignada en el lado de Rust
                free_string_ffi(resStruct.Value);

                return (result, isSuccess);
            }
            catch (Exception ex)
            {
                return ($"Error: {ex.Message}", false);
            }
            finally
            {
                if (inputPtr != IntPtr.Zero)
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        /// <summary>
        /// Envía una expresión matemática al motor de Rust para previsualización y retorna el resultado evaluado sin guardar en 'ans'.
        /// </summary>
        public static (string Result, bool IsSuccess) ProcessExpressionPreview(string expression)
        {
            if (string.IsNullOrWhiteSpace(expression))
            {
                return ("Error: Expresión vacía", false);
            }

            IntPtr inputPtr = IntPtr.Zero;
            try
            {
                byte[] utf8Bytes = Encoding.UTF8.GetBytes(expression + "\0");
                inputPtr = Marshal.AllocHGlobal(utf8Bytes.Length);
                Marshal.Copy(utf8Bytes, 0, inputPtr, utf8Bytes.Length);

                CalculatorResult resStruct = process_expression_preview_ffi(inputPtr);

                if (resStruct.Value == IntPtr.Zero)
                {
                    return ("Error: Resultado nulo", false);
                }

                string result = Marshal.PtrToStringUTF8(resStruct.Value) ?? "Error: Conversión de texto fallida";
                bool isSuccess = resStruct.IsSuccess != 0;

                free_string_ffi(resStruct.Value);

                return (result, isSuccess);
            }
            catch (Exception ex)
            {
                return ($"Error: {ex.Message}", false);
            }
            finally
            {
                if (inputPtr != IntPtr.Zero)
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        /// <summary>
        /// Valida de forma segura si una expresión es sintácticamente correcta en Rust.
        /// </summary>
        public static bool ValidateExpression(string expression)
        {
            if (string.IsNullOrWhiteSpace(expression))
            {
                return false;
            }

            IntPtr inputPtr = IntPtr.Zero;
            try
            {
                byte[] utf8Bytes = Encoding.UTF8.GetBytes(expression + "\0");
                inputPtr = Marshal.AllocHGlobal(utf8Bytes.Length);
                Marshal.Copy(utf8Bytes, 0, inputPtr, utf8Bytes.Length);

                byte isValid = validate_expression_ffi(inputPtr);
                return isValid != 0;
            }
            catch
            {
                return false;
            }
            finally
            {
                if (inputPtr != IntPtr.Zero)
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        /// <summary>
        /// Llama a Rust para realizar la conversión de unidades para una categoría física determinada (longitud, temperatura, masa o volumen).
        /// </summary>
        public static double ConvertUnits(double value, string fromUnit, string toUnit, string category)
        {
            if (string.IsNullOrEmpty(fromUnit) || string.IsNullOrEmpty(toUnit) || string.IsNullOrEmpty(category))
            {
                return 0.0;
            }

            IntPtr fromPtr = IntPtr.Zero;
            IntPtr toPtr = IntPtr.Zero;
            IntPtr catPtr = IntPtr.Zero;
            try
            {
                byte[] fromBytes = Encoding.UTF8.GetBytes(fromUnit + "\0");
                fromPtr = Marshal.AllocHGlobal(fromBytes.Length);
                Marshal.Copy(fromBytes, 0, fromPtr, fromBytes.Length);

                byte[] toBytes = Encoding.UTF8.GetBytes(toUnit + "\0");
                toPtr = Marshal.AllocHGlobal(toBytes.Length);
                Marshal.Copy(toBytes, 0, toPtr, toBytes.Length);

                byte[] catBytes = Encoding.UTF8.GetBytes(category + "\0");
                catPtr = Marshal.AllocHGlobal(catBytes.Length);
                Marshal.Copy(catBytes, 0, catPtr, catBytes.Length);

                return convert_units_ffi(value, fromPtr, toPtr, catPtr);
            }
            catch
            {
                return 0.0;
            }
            finally
            {
                if (fromPtr != IntPtr.Zero) Marshal.FreeHGlobal(fromPtr);
                if (toPtr != IntPtr.Zero) Marshal.FreeHGlobal(toPtr);
                if (catPtr != IntPtr.Zero) Marshal.FreeHGlobal(catPtr);
            }
        }

        /// <summary>
        /// Evalúa una expresión que contiene una variable reemplazando la variable por un valor concreto (utilizado para graficar).
        /// </summary>
        public static double EvaluateWithVar(string expression, double xValue)
        {
            if (string.IsNullOrWhiteSpace(expression))
            {
                return double.NaN;
            }

            IntPtr inputPtr = IntPtr.Zero;
            try
            {
                byte[] utf8Bytes = Encoding.UTF8.GetBytes(expression + "\0");
                inputPtr = Marshal.AllocHGlobal(utf8Bytes.Length);
                Marshal.Copy(utf8Bytes, 0, inputPtr, utf8Bytes.Length);

                return evaluate_with_var_ffi(inputPtr, xValue);
            }
            catch
            {
                return double.NaN;
            }
            finally
            {
                if (inputPtr != IntPtr.Zero)
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        /// <summary>
        /// Evalúa una expresión paramétrica sustituyendo la variable t por un valor.
        /// </summary>
        public static double EvaluateParametric(string expression, double tValue)
        {
            if (string.IsNullOrWhiteSpace(expression))
            {
                return double.NaN;
            }

            IntPtr inputPtr = IntPtr.Zero;
            try
            {
                byte[] utf8Bytes = Encoding.UTF8.GetBytes(expression + "\0");
                inputPtr = Marshal.AllocHGlobal(utf8Bytes.Length);
                Marshal.Copy(utf8Bytes, 0, inputPtr, utf8Bytes.Length);

                return evaluate_parametric_ffi(inputPtr, tValue);
            }
            catch
            {
                return double.NaN;
            }
            finally
            {
                if (inputPtr != IntPtr.Zero)
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        /// <summary>
        /// Evalúa una expresión polar sustituyendo la variable theta/θ por un valor.
        /// </summary>
        public static double EvaluatePolar(string expression, double thetaValue)
        {
            if (string.IsNullOrWhiteSpace(expression))
            {
                return double.NaN;
            }

            IntPtr inputPtr = IntPtr.Zero;
            try
            {
                byte[] utf8Bytes = Encoding.UTF8.GetBytes(expression + "\0");
                inputPtr = Marshal.AllocHGlobal(utf8Bytes.Length);
                Marshal.Copy(utf8Bytes, 0, inputPtr, utf8Bytes.Length);

                return evaluate_polar_ffi(inputPtr, thetaValue);
            }
            catch
            {
                return double.NaN;
            }
            finally
            {
                if (inputPtr != IntPtr.Zero)
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        /// <summary>
        /// Detecta la notación de la expresión ingresada: 1 para Infix (Infija), 2 para RPN (Posfija), 0 para vacía o inválida.
        /// </summary>
        public static int DetectNotation(string expression)
        {
            if (string.IsNullOrWhiteSpace(expression))
            {
                return 0;
            }

            IntPtr inputPtr = IntPtr.Zero;
            try
            {
                byte[] utf8Bytes = Encoding.UTF8.GetBytes(expression + "\0");
                inputPtr = Marshal.AllocHGlobal(utf8Bytes.Length);
                Marshal.Copy(utf8Bytes, 0, inputPtr, utf8Bytes.Length);

                return detect_notation_ffi(inputPtr);
            }
            catch
            {
                return 0;
            }
            finally
            {
                if (inputPtr != IntPtr.Zero)
                {
                    Marshal.FreeHGlobal(inputPtr);
                }
            }
        }

        /// <summary>
        /// Solicita a Rust el cálculo del paso adaptable de rejilla para la visualización del gráfico de funciones.
        /// </summary>
        public static double CalculateGridStep(double minVal, double maxVal)
        {
            try
            {
                return calculate_grid_step_ffi(minVal, maxVal);
            }
            catch
            {
                return 1.0;
            }
        }
    }
}
