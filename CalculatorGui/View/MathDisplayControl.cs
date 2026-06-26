using System;
using System.Collections.Generic;
using System.Text;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;

namespace CalculatorGui.View
{
    public class MathDisplayControl : ContentControl
    {
        public static readonly DependencyProperty ExpressionStringProperty =
            DependencyProperty.Register(
                nameof(ExpressionString),
                typeof(string),
                typeof(MathDisplayControl),
                new FrameworkPropertyMetadata(string.Empty, FrameworkPropertyMetadataOptions.AffectsMeasure | FrameworkPropertyMetadataOptions.AffectsRender, OnExpressionStringChanged));

        public string ExpressionString
        {
            get => (string)GetValue(ExpressionStringProperty);
            set => SetValue(ExpressionStringProperty, value);
        }

        private static void OnExpressionStringChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            if (d is MathDisplayControl control)
            {
                control.RebuildVisualTree();
            }
        }

        protected override void OnPropertyChanged(DependencyPropertyChangedEventArgs e)
        {
            base.OnPropertyChanged(e);
            if (e.Property == ForegroundProperty || e.Property == FontSizeProperty || e.Property == FontFamilyProperty)
            {
                RebuildVisualTree();
            }
        }

        public MathDisplayControl()
        {
            HorizontalContentAlignment = HorizontalAlignment.Right;
            VerticalContentAlignment = VerticalAlignment.Center;
        }

        private void RebuildVisualTree()
        {
            string expr = ExpressionString;
            if (string.IsNullOrEmpty(expr))
            {
                Content = null;
                return;
            }

            try
            {
                var tokens = MathTokenizer.Tokenize(expr);
                var parser = new MathParser(tokens);
                var element = parser.Parse();

                if (element != null)
                {
                    var visualTree = element.Render(Foreground ?? Brushes.White, FontSize, FontFamily ?? new FontFamily("Segoe UI"));
                    Content = visualTree;
                }
                else
                {
                    ShowFallbackText(expr);
                }
            }
            catch
            {
                ShowFallbackText(expr);
            }
        }

        private void ShowFallbackText(string text)
        {
            string clean = text.Replace("*", "×").Replace("pi", "π").Replace("theta", "θ").Replace("sqrt", "√");
            Content = new TextBlock
            {
                Text = clean,
                Foreground = Foreground ?? Brushes.White,
                FontSize = FontSize,
                FontFamily = FontFamily ?? new FontFamily("Segoe UI"),
                TextAlignment = TextAlignment.Right,
                VerticalAlignment = VerticalAlignment.Center,
                TextWrapping = TextWrapping.Wrap
            };
        }
    }

    #region Tokenizer and Parser

    public enum MathTokenType
    {
        Number,
        Identifier,
        Plus,
        Minus,
        Mul,
        Div,
        Power,
        LParen,
        RParen,
        Equal,
        Comma,
        Semicolon,
        LBracket,
        RBracket,
        Sqrt,
        Unknown
    }

    public class MathToken
    {
        public MathTokenType Type { get; set; }
        public string Value { get; set; } = string.Empty;
    }

    public static class MathTokenizer
    {
        public static List<MathToken> Tokenize(string expr)
        {
            var tokens = new List<MathToken>();
            int i = 0;
            while (i < expr.Length)
            {
                char c = expr[i];
                if (char.IsWhiteSpace(c))
                {
                    i++;
                    continue;
                }

                if (c == '+') { tokens.Add(new MathToken { Type = MathTokenType.Plus, Value = "+" }); i++; }
                else if (c == '-') { tokens.Add(new MathToken { Type = MathTokenType.Minus, Value = "-" }); i++; }
                else if (c == '*') { tokens.Add(new MathToken { Type = MathTokenType.Mul, Value = "*" }); i++; }
                else if (c == '/') { tokens.Add(new MathToken { Type = MathTokenType.Div, Value = "/" }); i++; }
                else if (c == '^') { tokens.Add(new MathToken { Type = MathTokenType.Power, Value = "^" }); i++; }
                else if (c == '(') { tokens.Add(new MathToken { Type = MathTokenType.LParen, Value = "(" }); i++; }
                else if (c == ')') { tokens.Add(new MathToken { Type = MathTokenType.RParen, Value = ")" }); i++; }
                else if (c == '=') { tokens.Add(new MathToken { Type = MathTokenType.Equal, Value = "=" }); i++; }
                else if (c == ',') { tokens.Add(new MathToken { Type = MathTokenType.Comma, Value = "," }); i++; }
                else if (c == ';') { tokens.Add(new MathToken { Type = MathTokenType.Semicolon, Value = ";" }); i++; }
                else if (c == '[') { tokens.Add(new MathToken { Type = MathTokenType.LBracket, Value = "[" }); i++; }
                else if (c == ']') { tokens.Add(new MathToken { Type = MathTokenType.RBracket, Value = "]" }); i++; }
                else if (c == '√') { tokens.Add(new MathToken { Type = MathTokenType.Sqrt, Value = "√" }); i++; }
                else if (char.IsDigit(c) || c == '.')
                {
                    var sb = new StringBuilder();
                    while (i < expr.Length && (char.IsDigit(expr[i]) || expr[i] == '.'))
                    {
                        sb.Append(expr[i]);
                        i++;
                    }
                    tokens.Add(new MathToken { Type = MathTokenType.Number, Value = sb.ToString() });
                }
                else if (char.IsLetter(c))
                {
                    var sb = new StringBuilder();
                    while (i < expr.Length && (char.IsLetter(expr[i]) || char.IsDigit(expr[i])))
                    {
                        sb.Append(expr[i]);
                        i++;
                    }
                    string val = sb.ToString();
                    if (val == "sqrt")
                    {
                        tokens.Add(new MathToken { Type = MathTokenType.Sqrt, Value = "sqrt" });
                    }
                    else
                    {
                        tokens.Add(new MathToken { Type = MathTokenType.Identifier, Value = val });
                    }
                }
                else
                {
                    tokens.Add(new MathToken { Type = MathTokenType.Unknown, Value = c.ToString() });
                    i++;
                }
            }
            return tokens;
        }
    }

    public class MathParser
    {
        private readonly List<MathToken> _tokens;
        private int _pos;

        public MathParser(List<MathToken> tokens)
        {
            _tokens = tokens;
            _pos = 0;
        }

        private MathToken? Peek() => _pos < _tokens.Count ? _tokens[_pos] : null;
        private MathToken? Read() => _pos < _tokens.Count ? _tokens[_pos++] : null;

        public MathElement? Parse()
        {
            var elements = new List<MathElement>();
            while (_pos < _tokens.Count)
            {
                var eqExpr = ParseEquation();
                if (eqExpr != null)
                {
                    elements.Add(eqExpr);
                }
                else
                {
                    break;
                }
            }

            if (elements.Count == 0) return null;
            if (elements.Count == 1) return elements[0];
            return new RowElement { Children = elements };
        }

        private MathElement? ParseEquation()
        {
            var left = ParseExpression();
            if (Peek()?.Type == MathTokenType.Equal)
            {
                Read(); // Consume '='
                var right = ParseExpression();
                var row = new RowElement();
                if (left != null) row.Children.Add(left);
                row.Children.Add(new OperatorElement { Op = "=" });
                if (right != null) row.Children.Add(right);
                return row;
            }
            return left;
        }

        private MathElement? ParseExpression()
        {
            var left = ParseTerm();
            while (true)
            {
                var next = Peek();
                if (next != null && (next.Type == MathTokenType.Plus || next.Type == MathTokenType.Minus))
                {
                    var opToken = Read()!;
                    var right = ParseTerm();
                    var row = new RowElement();
                    
                    if (left is RowElement leftRow)
                        row.Children.AddRange(leftRow.Children);
                    else if (left != null)
                        row.Children.Add(left);

                    row.Children.Add(new OperatorElement { Op = opToken.Value });

                    if (right is RowElement rightRow)
                        row.Children.AddRange(rightRow.Children);
                    else if (right != null)
                        row.Children.Add(right);

                    left = row;
                }
                else
                {
                    break;
                }
            }
            return left;
        }

        private MathElement? ParseTerm()
        {
            var left = ParseFactor();
            while (true)
            {
                var next = Peek();
                if (next != null && (next.Type == MathTokenType.Mul || next.Type == MathTokenType.Div))
                {
                    var opToken = Read()!;
                    var right = ParseFactor();
                    if (opToken.Type == MathTokenType.Div)
                    {
                        left = new FractionElement { Numerator = left, Denominator = right };
                    }
                    else
                    {
                        var row = new RowElement();
                        if (left is RowElement leftRow)
                            row.Children.AddRange(leftRow.Children);
                        else if (left != null)
                            row.Children.Add(left);

                        row.Children.Add(new OperatorElement { Op = "×" });

                        if (right is RowElement rightRow)
                            row.Children.AddRange(rightRow.Children);
                        else if (right != null)
                            row.Children.Add(right);

                        left = row;
                    }
                }
                else if (next != null && (next.Type == MathTokenType.LParen || next.Type == MathTokenType.Sqrt || next.Type == MathTokenType.Identifier || next.Type == MathTokenType.Number))
                {
                    // Implicit multiplication
                    var right = ParseFactor();
                    var row = new RowElement();
                    if (left is RowElement leftRow)
                        row.Children.AddRange(leftRow.Children);
                    else if (left != null)
                        row.Children.Add(left);

                    if (right is RowElement rightRow)
                        row.Children.AddRange(rightRow.Children);
                    else if (right != null)
                        row.Children.Add(right);

                    left = row;
                }
                else
                {
                    break;
                }
            }
            return left;
        }

        private MathElement? ParseFactor()
        {
            var left = ParsePrimary();
            if (Peek()?.Type == MathTokenType.Power)
            {
                Read(); // Consume '^'
                var right = ParseFactor();
                return new PowerElement { Base = left, Exponent = right };
            }
            return left;
        }

        private MathElement? ParsePrimary()
        {
            var next = Peek();
            if (next == null) return null;

            if (next.Type == MathTokenType.Number)
            {
                return new TextElement { Text = Read()!.Value };
            }
            if (next.Type == MathTokenType.Minus)
            {
                Read(); // Consume '-'
                var right = ParsePrimary();
                var row = new RowElement();
                row.Children.Add(new TextElement { Text = "-" });
                if (right != null) row.Children.Add(right);
                return row;
            }
            if (next.Type == MathTokenType.Plus)
            {
                Read(); // Consume '+'
                var right = ParsePrimary();
                var row = new RowElement();
                row.Children.Add(new TextElement { Text = "+" });
                if (right != null) row.Children.Add(right);
                return row;
            }
            if (next.Type == MathTokenType.LParen)
            {
                Read(); // Consume '('
                var inner = ParseExpression();
                if (Peek()?.Type == MathTokenType.RParen)
                {
                    Read(); // Consume ')'
                }
                return new ParenthesesElement { Inner = inner };
            }
            if (next.Type == MathTokenType.Sqrt)
            {
                Read(); // Consume 'sqrt' or '√'
                if (Peek()?.Type == MathTokenType.LParen)
                {
                    Read(); // Consume '('
                    var inner = ParseExpression();
                    if (Peek()?.Type == MathTokenType.RParen)
                    {
                        Read(); // Consume ')'
                    }
                    return new SqrtElement { Inner = inner };
                }
                else
                {
                    var inner = ParsePrimary();
                    return new SqrtElement { Inner = inner };
                }
            }
            if (next.Type == MathTokenType.Identifier)
            {
                var idToken = Read()!;
                if (Peek()?.Type == MathTokenType.LParen)
                {
                    Read(); // Consume '('
                    var inner = ParseExpression();
                    if (Peek()?.Type == MathTokenType.RParen)
                    {
                        Read(); // Consume ')'
                    }
                    var row = new RowElement();
                    row.Children.Add(new TextElement { Text = idToken.Value });
                    row.Children.Add(new ParenthesesElement { Inner = inner });
                    return row;
                }

                string text = idToken.Value;
                if (text.Equals("pi", StringComparison.OrdinalIgnoreCase)) text = "π";
                else if (text.Equals("theta", StringComparison.OrdinalIgnoreCase)) text = "θ";

                return new TextElement { Text = text, IsItalic = true };
            }
            if (next.Type == MathTokenType.LBracket)
            {
                var sb = new StringBuilder();
                while (_pos < _tokens.Count)
                {
                    var tok = Read()!;
                    sb.Append(tok.Value);
                    if (tok.Type == MathTokenType.RBracket)
                        break;
                }
                return new TextElement { Text = sb.ToString() };
            }

            return new TextElement { Text = Read()!.Value };
        }
    }

    #endregion

    #region Layout elements

    public abstract class MathElement
    {
        public abstract UIElement Render(Brush foreground, double fontSize, FontFamily fontFamily);
    }

    public class TextElement : MathElement
    {
        public string Text { get; set; } = string.Empty;
        public bool IsItalic { get; set; }
        public bool IsBold { get; set; }

        public override UIElement Render(Brush foreground, double fontSize, FontFamily fontFamily)
        {
            return new TextBlock
            {
                Text = Text,
                Foreground = foreground,
                FontSize = fontSize,
                FontFamily = fontFamily,
                FontStyle = IsItalic ? FontStyles.Italic : FontStyles.Normal,
                FontWeight = IsBold ? FontWeights.Bold : FontWeights.Normal,
                VerticalAlignment = VerticalAlignment.Center
            };
        }
    }

    public class OperatorElement : MathElement
    {
        public string Op { get; set; } = string.Empty;

        public override UIElement Render(Brush foreground, double fontSize, FontFamily fontFamily)
        {
            return new TextBlock
            {
                Text = " " + Op + " ",
                Foreground = foreground,
                FontSize = fontSize,
                FontFamily = fontFamily,
                VerticalAlignment = VerticalAlignment.Center
            };
        }
    }

    public class FractionElement : MathElement
    {
        public MathElement? Numerator { get; set; }
        public MathElement? Denominator { get; set; }

        public override UIElement Render(Brush foreground, double fontSize, FontFamily fontFamily)
        {
            double subFontSize = fontSize * 0.82;
            var numUI = Numerator?.Render(foreground, subFontSize, fontFamily) ?? new TextBlock();
            var denUI = Denominator?.Render(foreground, subFontSize, fontFamily) ?? new TextBlock();

            var grid = new Grid();
            grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
            grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
            grid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });

            Grid.SetRow(numUI, 0);
            grid.Children.Add(numUI);

            var line = new Border
            {
                Height = 1.5,
                Background = foreground,
                HorizontalAlignment = HorizontalAlignment.Stretch,
                Margin = new Thickness(2, 2, 2, 2)
            };
            Grid.SetRow(line, 1);
            grid.Children.Add(line);

            Grid.SetRow(denUI, 2);
            grid.Children.Add(denUI);

            numUI.SetValue(FrameworkElement.HorizontalAlignmentProperty, HorizontalAlignment.Center);
            denUI.SetValue(FrameworkElement.HorizontalAlignmentProperty, HorizontalAlignment.Center);

            var container = new Grid
            {
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(4, 0, 4, 0)
            };
            container.Children.Add(grid);
            return container;
        }
    }

    public class PowerElement : MathElement
    {
        public MathElement? Base { get; set; }
        public MathElement? Exponent { get; set; }

        public override UIElement Render(Brush foreground, double fontSize, FontFamily fontFamily)
        {
            var baseUI = Base?.Render(foreground, fontSize, fontFamily) ?? new TextBlock();
            var expUI = Exponent?.Render(foreground, fontSize * 0.65, fontFamily) ?? new TextBlock();

            var stack = new StackPanel
            {
                Orientation = Orientation.Horizontal,
                VerticalAlignment = VerticalAlignment.Center
            };

            baseUI.SetValue(FrameworkElement.VerticalAlignmentProperty, VerticalAlignment.Center);
            stack.Children.Add(baseUI);

            var expContainer = new Grid
            {
                VerticalAlignment = VerticalAlignment.Top,
                Margin = new Thickness(1, 0, 0, fontSize * 0.4)
            };
            expContainer.Children.Add(expUI);
            stack.Children.Add(expContainer);

            return stack;
        }
    }

    public class SqrtElement : MathElement
    {
        public MathElement? Inner { get; set; }

        public override UIElement Render(Brush foreground, double fontSize, FontFamily fontFamily)
        {
            var innerUI = Inner?.Render(foreground, fontSize, fontFamily) ?? new TextBlock();

            var grid = new Grid { VerticalAlignment = VerticalAlignment.Center };
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });

            var radical = new TextBlock
            {
                Text = "√",
                Foreground = foreground,
                FontSize = fontSize * 1.1,
                FontFamily = fontFamily,
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(2, 0, 0, 0)
            };
            Grid.SetColumn(radical, 0);
            grid.Children.Add(radical);

            var innerGrid = new Grid();
            innerGrid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
            innerGrid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });

            var topLine = new Border
            {
                Height = 1.2,
                Background = foreground,
                HorizontalAlignment = HorizontalAlignment.Stretch,
                VerticalAlignment = VerticalAlignment.Bottom,
                Margin = new Thickness(0, 0, 0, 1)
            };
            Grid.SetRow(topLine, 0);
            innerGrid.Children.Add(topLine);

            Grid.SetRow(innerUI, 1);
            innerGrid.Children.Add(innerUI);

            Grid.SetColumn(innerGrid, 1);
            grid.Children.Add(innerGrid);

            return grid;
        }
    }

    public class ParenthesesElement : MathElement
    {
        public MathElement? Inner { get; set; }

        public override UIElement Render(Brush foreground, double fontSize, FontFamily fontFamily)
        {
            var innerUI = Inner?.Render(foreground, fontSize, fontFamily) ?? new TextBlock();

            var stack = new StackPanel
            {
                Orientation = Orientation.Horizontal,
                VerticalAlignment = VerticalAlignment.Center
            };

            var leftParen = new TextBlock
            {
                Text = "(",
                Foreground = foreground,
                FontSize = fontSize * 1.1,
                FontFamily = fontFamily,
                VerticalAlignment = VerticalAlignment.Center
            };
            stack.Children.Add(leftParen);

            stack.Children.Add(innerUI);

            var rightParen = new TextBlock
            {
                Text = ")",
                Foreground = foreground,
                FontSize = fontSize * 1.1,
                FontFamily = fontFamily,
                VerticalAlignment = VerticalAlignment.Center
            };
            stack.Children.Add(rightParen);

            return stack;
        }
    }

    public class RowElement : MathElement
    {
        public List<MathElement> Children { get; set; } = new List<MathElement>();

        public override UIElement Render(Brush foreground, double fontSize, FontFamily fontFamily)
        {
            var stack = new StackPanel
            {
                Orientation = Orientation.Horizontal,
                VerticalAlignment = VerticalAlignment.Center
            };

            foreach (var child in Children)
            {
                if (child != null)
                {
                    stack.Children.Add(child.Render(foreground, fontSize, fontFamily));
                }
            }

            return stack;
        }
    }

    #endregion
}
