using System;
using System.Windows;
using System.Windows.Controls;

namespace CalculatorGui
{
    /// <summary>
    /// Interaction logic for MenuControl.xaml
    /// </summary>
    public partial class MenuControl : UserControl
    {
        public event EventHandler<string>? MenuOptionSelected;
        public event EventHandler? MenuToggleRequest;

        public MenuControl()
        {
            InitializeComponent();
        }

        private void BtnMenuToggle_Click(object sender, RoutedEventArgs e)
        {
            MenuToggleRequest?.Invoke(this, EventArgs.Empty);
        }

        private void NavButton_Click(object sender, RoutedEventArgs e)
        {
            if (sender is Button btn && btn.Tag != null)
            {
                string option = btn.Tag.ToString() ?? "";
                MenuOptionSelected?.Invoke(this, option);
            }
        }
    }
}
