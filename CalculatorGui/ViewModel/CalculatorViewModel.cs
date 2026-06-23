using System;
using System.Collections.ObjectModel;
using System.Windows.Input;
using CalculatorGui;

namespace CalculatorGui.ViewModel
{
    /// <summary>
    /// ViewModel para la vista de la calculadora.
    /// Maneja el estado de la expresión, los comandos de los botones y la comunicación con el motor Rust.
    /// </summary>
    public class CalculatorViewModel : ViewModelBase
    {
        private string _expression = "";
        private string _result = "0";
        private string _notationType = "Notación: Infix (Auto)";
        private double _previewOpacity = 0.5;
        private bool _isScientific = true;
        private bool _isSecondFunction = false;
        private bool _angleModeDegrees = false;
        private readonly ObservableCollection<HistoryItem> _history;

        public ObservableCollection<ButtonInfo> ScientificButtons { get; } = new ObservableCollection<ButtonInfo>();
        public ObservableCollection<ButtonInfo> StandardButtons { get; } = new ObservableCollection<ButtonInfo>();

        public CalculatorViewModel(ObservableCollection<HistoryItem> history)
        {
            _history = history;
            
            // Inicialización de comandos vinculados a los botones de la interfaz
            ClearCommand = new RelayCommand(_ => Clear());
            BackspaceCommand = new RelayCommand(_ => Backspace());
            CalculateCommand = new RelayCommand(_ => Calculate());
            ToggleSecondCommand = new RelayCommand(_ => IsSecondFunction = !IsSecondFunction);
            ToggleAngleModeCommand = new RelayCommand(_ => AngleModeDegrees = !AngleModeDegrees);
            InsertAnsCommand = new RelayCommand(_ => InsertAns());

            InitializeButtons();
        }

        private void InitializeButtons()
        {
            // Botones Científicos (5 columnas, 7 filas)
            // Fila 0
            ScientificButtons.Add(new ButtonInfo { NormalText = "2nd", Row = 0, Column = 0, StyleKey = "SciButtonStyle", SpecialType = "ToggleSecond", Command = ToggleSecondCommand });
            ScientificButtons.Add(new ButtonInfo { NormalText = "pi", ShiftText = "tau", Row = 0, Column = 1, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "e", ShiftText = "phi", Row = 0, Column = 2, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "C", Row = 0, Column = 3, StyleKey = "ActionButtonStyle", SpecialType = "Clear", Command = ClearCommand });
            ScientificButtons.Add(new ButtonInfo { NormalText = "⌫", Row = 0, Column = 4, StyleKey = "ActionButtonStyle", SpecialType = "Backspace" });

            // Fila 1
            ScientificButtons.Add(new ButtonInfo { NormalText = "x^2", ShiftText = "x^3", Row = 1, Column = 0, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "1/x", Row = 1, Column = 1, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "abs", Row = 1, Column = 2, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "exp", Row = 1, Column = 3, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "mod", ShiftText = "gcd", Row = 1, Column = 4, StyleKey = "SciButtonStyle" });

            // Fila 2
            ScientificButtons.Add(new ButtonInfo { NormalText = "sqrt", ShiftText = "sqrt2", Row = 2, Column = 0, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "(", Row = 2, Column = 1, StyleKey = "OpButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = ")", Row = 2, Column = 2, StyleKey = "OpButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "fact", ShiftText = "%", Row = 2, Column = 3, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "/", Row = 2, Column = 4, StyleKey = "OpButtonStyle" });

            // Fila 3
            ScientificButtons.Add(new ButtonInfo { NormalText = "^", ShiftText = "^2", Row = 3, Column = 0, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "7", Row = 3, Column = 1, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "8", Row = 3, Column = 2, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "9", Row = 3, Column = 3, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "*", Row = 3, Column = 4, StyleKey = "OpButtonStyle" });

            // Fila 4
            ScientificButtons.Add(new ButtonInfo { NormalText = "10^x", ShiftText = "2^x", Row = 4, Column = 0, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "4", Row = 4, Column = 1, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "5", Row = 4, Column = 2, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "6", Row = 4, Column = 3, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "-", Row = 4, Column = 4, StyleKey = "OpButtonStyle" });

            // Fila 5
            ScientificButtons.Add(new ButtonInfo { NormalText = "log", ShiftText = "log2", Row = 5, Column = 0, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "1", Row = 5, Column = 1, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "2", Row = 5, Column = 2, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "3", Row = 5, Column = 3, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "+", Row = 5, Column = 4, StyleKey = "OpButtonStyle" });

            // Fila 6
            ScientificButtons.Add(new ButtonInfo { NormalText = "ln", ShiftText = "exp", Row = 6, Column = 0, StyleKey = "SciButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "ANS", Row = 6, Column = 1, StyleKey = "SciButtonStyle", SpecialType = "Ans", Command = InsertAnsCommand });
            ScientificButtons.Add(new ButtonInfo { NormalText = "0", Row = 6, Column = 2, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = ".", Row = 6, Column = 3, StyleKey = "DigitButtonStyle" });
            ScientificButtons.Add(new ButtonInfo { NormalText = "=", Row = 6, Column = 4, StyleKey = "EqualsButtonStyle", SpecialType = "Equal", Command = CalculateCommand });


            // Botones Estándar (4 columnas, 6 filas)
            // Fila 0
            StandardButtons.Add(new ButtonInfo { NormalText = "%", Row = 0, Column = 0, StyleKey = "SciButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "CE", Row = 0, Column = 1, StyleKey = "ActionButtonStyle", SpecialType = "Clear", Command = ClearCommand });
            StandardButtons.Add(new ButtonInfo { NormalText = "C", Row = 0, Column = 2, StyleKey = "ActionButtonStyle", SpecialType = "Clear", Command = ClearCommand });
            StandardButtons.Add(new ButtonInfo { NormalText = "⌫", Row = 0, Column = 3, StyleKey = "ActionButtonStyle", SpecialType = "Backspace" });

            // Fila 1
            StandardButtons.Add(new ButtonInfo { NormalText = "1/x", Row = 1, Column = 0, StyleKey = "SciButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "x^2", Row = 1, Column = 1, StyleKey = "SciButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "sqrt", Row = 1, Column = 2, StyleKey = "SciButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "/", Row = 1, Column = 3, StyleKey = "OpButtonStyle" });

            // Fila 2
            StandardButtons.Add(new ButtonInfo { NormalText = "7", Row = 2, Column = 0, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "8", Row = 2, Column = 1, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "9", Row = 2, Column = 2, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "*", Row = 2, Column = 3, StyleKey = "OpButtonStyle" });

            // Fila 3
            StandardButtons.Add(new ButtonInfo { NormalText = "4", Row = 3, Column = 0, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "5", Row = 3, Column = 1, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "6", Row = 3, Column = 2, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "-", Row = 3, Column = 3, StyleKey = "OpButtonStyle" });

            // Fila 4
            StandardButtons.Add(new ButtonInfo { NormalText = "1", Row = 4, Column = 0, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "2", Row = 4, Column = 1, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "3", Row = 4, Column = 2, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "+", Row = 4, Column = 3, StyleKey = "OpButtonStyle" });

            // Fila 5
            StandardButtons.Add(new ButtonInfo { NormalText = "+/-", Row = 5, Column = 0, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "0", Row = 5, Column = 1, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = ".", Row = 5, Column = 2, StyleKey = "DigitButtonStyle" });
            StandardButtons.Add(new ButtonInfo { NormalText = "=", Row = 5, Column = 3, StyleKey = "EqualsButtonStyle", SpecialType = "Equal", Command = CalculateCommand });
        }

        // Expresión matemática que el usuario está escribiendo
        public string Expression
        {
            get => _expression;
            set
            {
                if (SetProperty(ref _expression, value))
                {
                    // Valida la expresión en tiempo real mientras el usuario escribe
                    UpdateValidationStatus();
                }
            }
        }

        // Resultado de la expresión o vista previa (preview) del resultado
        public string Result
        {
            get => _result;
            set => SetProperty(ref _result, value);
        }

        // Tipo de notación detectada por Rust (Infija o RPN)
        public string NotationType
        {
            get => _notationType;
            set => SetProperty(ref _notationType, value);
        }

        // Opacidad del texto de vista previa (para indicar si es resultado final o preliminar)
        public double PreviewOpacity
        {
            get => _previewOpacity;
            set => SetProperty(ref _previewOpacity, value);
        }

        public bool IsScientific
        {
            get => _isScientific;
            set => SetProperty(ref _isScientific, value);
        }

        // Indica si las segundas funciones científicas están activas (2nd)
        public bool IsSecondFunction
        {
            get => _isSecondFunction;
            set
            {
                if (SetProperty(ref _isSecondFunction, value))
                {
                    foreach (var btn in ScientificButtons)
                    {
                        if (btn.SpecialType == "ToggleSecond")
                        {
                            btn.IsActive = value;
                        }
                    }
                }
            }
        }

        // Modo de medición de ángulos: false = Radianes (RAD), true = Grados (DEG)
        public bool AngleModeDegrees
        {
            get => _angleModeDegrees;
            set
            {
                if (SetProperty(ref _angleModeDegrees, value))
                {
                    // Actualiza el modo en el motor Rust
                    CalculatorBridge.SetAngleMode(value);
                    OnPropertyChanged(nameof(AngleModeText));
                    UpdateValidationStatus();
                }
            }
        }

        public string AngleModeText => AngleModeDegrees ? "DEG" : "RAD";

        public ICommand ClearCommand { get; }
        public ICommand BackspaceCommand { get; }
        public ICommand CalculateCommand { get; }
        public ICommand ToggleSecondCommand { get; }
        public ICommand ToggleAngleModeCommand { get; }
        public ICommand InsertAnsCommand { get; }

        // Inserta la constante del último resultado obtenido (ANS)
        private void InsertAns()
        {
            Expression += "ans";
        }

        // Borra la entrada de texto y restablece el resultado
        private void Clear()
        {
            Expression = "";
            Result = "0";
        }

        // Elimina el último carácter de la expresión
        private void Backspace()
        {
            if (Expression.Length > 0)
            {
                Expression = Expression.Substring(0, Expression.Length - 1);
            }
        }

        // Realiza el cálculo final al presionar el botón '='
        public void Calculate()
        {
            string expr = Expression.Trim();
            if (string.IsNullOrEmpty(expr)) return;

            // Envía la expresión al motor Rust a través del puente FFI
            var (res, isSuccess) = CalculatorBridge.ProcessExpression(expr);
            Result = res;
            PreviewOpacity = 1.0; // Opacidad total indicando resultado final

            if (isSuccess)
            {
                // Guarda en el historial de cálculos de la aplicación
                _history.Insert(0, new HistoryItem
                {
                    Expression = expr,
                    Result = res
                });
            }
        }

        // Realiza una validación rápida y cálculo preliminar en tiempo real
        private void UpdateValidationStatus()
        {
            string text = Expression.Trim();
            if (string.IsNullOrEmpty(text))
            {
                Result = "0";
                PreviewOpacity = 0.5;
                UpdateNotationType();
                return;
            }

            // Pregunta al motor Rust si la expresión actual es válida
            bool isValid = CalculatorBridge.ValidateExpression(text);
            if (isValid)
            {
                var (res, isSuccess) = CalculatorBridge.ProcessExpressionPreview(text);
                if (isSuccess)
                {
                    Result = res;
                    PreviewOpacity = 0.6; // Muestra un resultado preliminar tenue
                }
            }
            else
            {
                PreviewOpacity = 0.3; // Opacidad muy baja si tiene errores de sintaxis
                Result = ""; // Borra el resultado previo obsoleto al haber error
            }

            UpdateNotationType();
        }

        // Detecta automáticamente si el usuario escribe en notación infija tradicional o en RPN
        private void UpdateNotationType()
        {
            string text = Expression.Trim();
            if (string.IsNullOrEmpty(text))
            {
                NotationType = "Notación: Infix (Auto)";
                return;
            }

            int notation = CalculatorBridge.DetectNotation(text);
            if (notation == 2)
            {
                NotationType = "Notación: RPN (Posfija)";
            }
            else
            {
                NotationType = "Notación: Infix (Auto)";
            }
        }
    }
}
