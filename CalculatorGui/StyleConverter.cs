using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;

namespace CalculatorGui
{
    public class StyleConverter : IValueConverter
    {
        public object? Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is string styleKey && !string.IsNullOrEmpty(styleKey))
            {
                return Application.Current.TryFindResource(styleKey);
            }
            return null;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}
