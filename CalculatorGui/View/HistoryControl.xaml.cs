using System;
using System.Collections;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;

namespace CalculatorGui
{
    /// <summary>
    /// Interaction logic for HistoryControl.xaml
    /// </summary>
    public partial class HistoryControl : UserControl
    {
        public event EventHandler<HistoryItem>? HistoryItemSelect;
        public event EventHandler? HistoryCleared;

        public HistoryControl()
        {
            InitializeComponent();
        }

        public IEnumerable ItemsSource
        {
            get => LstHistory.ItemsSource;
            set => LstHistory.ItemsSource = value;
        }

        private void LstHistory_MouseDoubleClick(object sender, MouseButtonEventArgs e)
        {
            if (LstHistory.SelectedItem is HistoryItem selectedItem)
            {
                HistoryItemSelect?.Invoke(this, selectedItem);
            }
        }

        private void ClearHistory_Click(object sender, RoutedEventArgs e)
        {
            HistoryCleared?.Invoke(this, EventArgs.Empty);
        }
    }
}
