using System;
using System.Globalization;
using System.Windows.Data;

namespace CalculatorGui
{
    public class ButtonTextConverter : IMultiValueConverter
    {
        public object Convert(object[] values, Type targetType, object parameter, CultureInfo culture)
        {
            if (values.Length >= 3 && values[0] is string normalText && values[1] is string shiftText && values[2] is bool isSecond)
            {
                return (isSecond && !string.IsNullOrEmpty(shiftText)) ? shiftText : normalText;
            }
            return "";
        }

        public object[] ConvertBack(object value, Type[] targetTypes, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}
