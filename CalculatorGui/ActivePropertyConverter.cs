using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;
using System.Windows.Media;

namespace CalculatorGui
{
    public class ActivePropertyConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool isActive && isActive)
            {
                string propType = parameter as string ?? "Background";
                if (propType == "Background")
                {
                    return Application.Current.TryFindResource("AccentBrush") ?? Brushes.DeepSkyBlue;
                }
                else if (propType == "Foreground")
                {
                    return Brushes.White;
                }
            }
            return DependencyProperty.UnsetValue;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }
    }
}
