using System;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using CalculatorGui.ViewModel;

namespace CalculatorGui.View
{
    /// <summary>
    /// Interaction logic for CalculatorView.xaml
    /// </summary>
    public partial class CalculatorView : UserControl
    {
        public CalculatorView()
        {
            InitializeComponent();
        }

        private CalculatorViewModel? ViewModel => DataContext as CalculatorViewModel;

        private void CalculatorButton_Click(object sender, RoutedEventArgs e)
        {
            if (sender is Button btn)
            {
                if (btn.DataContext is ButtonInfo info)
                {
                    if (info.SpecialType == "Backspace")
                    {
                        BackspaceButton_Click(sender, e);
                        return;
                    }

                    if (info.SpecialType == "Clear" || info.SpecialType == "Equal" || info.SpecialType == "ToggleSecond" || info.SpecialType == "Ans")
                    {
                        if (info.SpecialType == "Equal")
                        {
                            Keyboard.ClearFocus();
                        }
                        return;
                    }

                    // Obtiene el texto del botón según el estado de la segunda función
                    string content = (ViewModel?.IsSecondFunction == true && !string.IsNullOrEmpty(info.ShiftText))
                        ? info.ShiftText
                        : info.NormalText;

                    if (content == "+/-")
                    {
                        ToggleSign();
                        return;
                    }

                    if (IsUnaryFunction(content))
                    {
                        ApplyUnaryFunction(content);
                        return;
                    }

                    string[] functions = { "sin", "cos", "tan", "ctan", "sinh", "cosh", "tanh", "asin", "acos", "atan", "asinh", "acosh", "atanh", "log", "log2", "ln", "exp", "floor", "ceil", "round", "trunc", "abs", "sqrt", "min", "max", "mod", "gcd", "nCr", "nPr", "fact" };
                    if (Array.Exists(functions, f => f == content))
                    {
                        AppendText(content + "(");
                    }
                    else
                    {
                        AppendText(content);
                    }
                }
                else if (btn.Content != null)
                {
                    string content = btn.Content.ToString() ?? "";
                    if (content == "C" || content == "CE" || content == "⌫" || content == "=" || content == "ANS" || content == "2nd" || content == "Resolver Ecuación" || content == "DEG" || content == "RAD")
                    {
                        if (content == "=")
                        {
                            Keyboard.ClearFocus();
                        }
                        return;
                    }

                    if (content == "+/-")
                    {
                        ToggleSign();
                        return;
                    }

                    if (IsUnaryFunction(content))
                    {
                        ApplyUnaryFunction(content);
                        return;
                    }

                    string[] functions = { "sin", "cos", "tan", "ctan", "sinh", "cosh", "tanh", "asin", "acos", "atan", "asinh", "acosh", "atanh", "log", "log2", "ln", "exp", "floor", "ceil", "round", "trunc", "abs", "sqrt", "min", "max", "mod", "gcd", "nCr", "nPr", "fact" };
                    if (Array.Exists(functions, f => f == content))
                    {
                        AppendText(content + "(");
                    }
                    else
                    {
                        AppendText(content);
                    }
                }
            }
        }

        private void BtnTrig_Click(object sender, RoutedEventArgs e)
        {
            PopupTrig.IsOpen = true;
        }

        private void BtnFunc_Click(object sender, RoutedEventArgs e)
        {
            PopupFunc.IsOpen = true;
        }

        private void PopupButton_Click(object sender, RoutedEventArgs e)
        {
            if (sender is Button btn && btn.Content != null)
            {
                string functionName = btn.Content.ToString() ?? "";
                AppendText(functionName + "(");
                PopupTrig.IsOpen = false;
                PopupFunc.IsOpen = false;
            }
        }

        private void PopupButtonVariable_Click(object sender, RoutedEventArgs e)
        {
            AppendText("x");
            PopupFunc.IsOpen = false;
        }

        private void PopupButtonSolve_Click(object sender, RoutedEventArgs e)
        {
            SolveEqButton_Click(sender, e);
            PopupFunc.IsOpen = false;
        }

        private void MenuItem_Click(object sender, RoutedEventArgs e)
        {
            if (sender is MenuItem menuItem && menuItem.Header != null)
            {
                string functionName = menuItem.Header.ToString() ?? "";
                AppendText(functionName + "(");
            }
        }

        private void MenuItemVariable_Click(object sender, RoutedEventArgs e)
        {
            AppendText("x");
        }

        private void MenuItemSolve_Click(object sender, RoutedEventArgs e)
        {
            SolveEqButton_Click(sender, e);
        }

        private void ToggleSign()
        {
            string text = TxtExpression.Text;
            if (string.IsNullOrEmpty(text))
            {
                AppendText("-");
                return;
            }

            if (text.EndsWith(")"))
            {
                // Try to find if it matches "(-number)" at the end and toggle it back
                int lastOpen = text.LastIndexOf('(');
                if (lastOpen >= 0 && lastOpen < text.Length - 2 && text[lastOpen + 1] == '-')
                {
                    // It starts with (- and ends with ). Let's unwrap it
                    string numberPart = text.Substring(lastOpen + 2, text.Length - 1 - (lastOpen + 2));
                    TxtExpression.Text = text.Substring(0, lastOpen) + numberPart;
                    TxtExpression.CaretIndex = TxtExpression.Text.Length;
                    TxtExpression.Focus();
                    return;
                }
            }

            // Otherwise, find the last number or parameter and negate it
            int i = text.Length - 1;
            while (i >= 0 && (char.IsDigit(text[i]) || text[i] == '.'))
            {
                i--;
            }

            int numStart = i + 1;
            if (numStart < text.Length)
            {
                string numStr = text.Substring(numStart);
                TxtExpression.Text = text.Substring(0, numStart) + "(-" + numStr + ")";
                TxtExpression.CaretIndex = TxtExpression.Text.Length;
            }
            else
            {
                AppendText("-");
            }
            TxtExpression.Focus();
        }

        private void SolveEqButton_Click(object sender, RoutedEventArgs e)
        {
            if (ViewModel == null) return;
            if (!ViewModel.Expression.Contains("="))
            {
                AppendText(" = ");
            }
            else
            {
                AppendText("x");
            }
        }

        private void BackspaceButton_Click(object sender, RoutedEventArgs e)
        {
            int caretIndex = TxtExpression.CaretIndex;
            if (TxtExpression.Text.Length > 0 && caretIndex > 0)
            {
                TxtExpression.Text = TxtExpression.Text.Remove(caretIndex - 1, 1);
                TxtExpression.CaretIndex = caretIndex - 1;
            }
            TxtExpression.Focus();
        }

        private void AppendText(string text)
        {
            int caretIndex = TxtExpression.CaretIndex;
            TxtExpression.Text = TxtExpression.Text.Insert(caretIndex, text);
            TxtExpression.CaretIndex = caretIndex + text.Length;
            TxtExpression.Focus();
        }

        private bool IsUnaryFunction(string content)
        {
            return content == "sqrt" || content == "x^2" || content == "x^3" || content == "1/x" || content == "10^x" || content == "2^x" || content == "fact";
        }

        private void ApplyUnaryFunction(string content)
        {
            if (ViewModel == null) return;
            string currentExpr = TxtExpression.Text.Trim();
            if (string.IsNullOrEmpty(currentExpr))
            {
                currentExpr = "ans";
            }

            string newExpr = "";
            switch (content)
            {
                case "sqrt":
                    newExpr = $"sqrt({currentExpr})";
                    break;
                case "x^2":
                    newExpr = $"({currentExpr})^2";
                    break;
                case "x^3":
                    newExpr = $"({currentExpr})^3";
                    break;
                case "1/x":
                    newExpr = $"1/({currentExpr})";
                    break;
                case "10^x":
                    newExpr = $"10^({currentExpr})";
                    break;
                case "2^x":
                    newExpr = $"2^({currentExpr})";
                    break;
                case "fact":
                    newExpr = $"fact({currentExpr})";
                    break;
            }

            TxtExpression.Text = newExpr;
            TxtExpression.CaretIndex = newExpr.Length;
            TxtExpression.Focus();

            ViewModel.Calculate();
        }

        private void TxtExpression_GotFocus(object sender, RoutedEventArgs e)
        {
            // Focus helper
        }

        private void TxtExpression_PreviewKeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key == Key.Enter)
            {
                ViewModel?.CalculateCommand.Execute(null);
                Keyboard.ClearFocus();
                e.Handled = true;
            }
        }
    }
}
