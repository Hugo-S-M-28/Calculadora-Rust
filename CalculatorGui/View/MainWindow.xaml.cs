using System;
using System.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Input;
using System.Windows.Media.Animation;
using CalculatorGui.ViewModel;

namespace CalculatorGui
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        private const double MenuExpandedWidth = 220;
        private const double MenuAnimationDurationMs = 200;
        private const double HistoryExpandedStarWidth = 1.2;
        private const double HistoryAutoCloseThreshold = 760;

        private bool _isDarkMode = true;

        public MainWindow()
        {
            InitializeComponent();

            var vm = new MainViewModel();
            DataContext = vm;

            // Bind History control ItemsSource
            HistoryPanel.ItemsSource = vm.History;

            // Subscribe to ViewModel changes for UI animation & structural updates
            vm.PropertyChanged += MainViewModel_PropertyChanged;

            // Apply default theme and initial layout state
            ApplyTheme(_isDarkMode);
            ToggleMenu(vm.IsMenuOpen);

            // Re-position the floating menu when the window is resized so it
            // stretches to the full height of the client area.
            SizeChanged += (_, __) => LayoutOverlay();
            Loaded += (_, __) => LayoutOverlay();
        }

        // ===================== Event handlers =====================

        private void MainViewModel_PropertyChanged(object? sender, PropertyChangedEventArgs e)
        {
            if (DataContext is not MainViewModel vm) return;

            if (e.PropertyName == nameof(MainViewModel.IsMenuOpen))
            {
                ToggleMenu(vm.IsMenuOpen);
            }
        }

        private void MenuPanel_MenuOptionSelected(object sender, string mode)
        {
            if (DataContext is MainViewModel vm)
            {
                vm.NavigateCommand.Execute(mode);
            }
        }

        private void MenuPanel_MenuToggleRequest(object sender, EventArgs e)
        {
            if (DataContext is MainViewModel vm)
            {
                vm.ToggleMenuCommand.Execute(null);
            }
        }

        private void HistoryPanel_HistoryItemSelect(object sender, HistoryItem selectedItem)
        {
            if (DataContext is MainViewModel vm)
            {
                // Switch back to calculator if not active, keeping the same scientific state
                vm.NavigateCommand.Execute(vm.CalculatorVm.IsScientific ? "Scientific" : "Standard");
                vm.CalculatorVm.Expression = selectedItem.Expression;
                vm.CalculatorVm.Result = selectedItem.Result;
                vm.CalculatorVm.PreviewOpacity = 1.0;
            }
        }

        private void HistoryPanel_HistoryCleared(object sender, EventArgs e)
        {
            if (DataContext is MainViewModel vm)
            {
                vm.History.Clear();
            }
        }

        private void BtnMenuToggle_Click(object sender, RoutedEventArgs e)
        {
            if (DataContext is MainViewModel vm)
            {
                vm.ToggleMenuCommand.Execute(null);
            }
        }

        private void OverlayLayer_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            if (DataContext is MainViewModel vm && vm.IsMenuOpen)
            {
                if (e.OriginalSource == OverlayLayer)
                {
                    vm.IsMenuOpen = false;
                }
            }
        }

        protected override void OnRenderSizeChanged(SizeChangedInfo sizeInfo)
        {
            base.OnRenderSizeChanged(sizeInfo);
            if (DataContext is MainViewModel vm)
            {
                if (sizeInfo.NewSize.Width < HistoryAutoCloseThreshold && vm.IsHistoryOpen)
                {
                    vm.IsHistoryOpen = false;
                }
            }
        }

        private void BtnThemeToggle_Click(object sender, RoutedEventArgs e)
        {
            _isDarkMode = !_isDarkMode;
            ApplyTheme(_isDarkMode);
        }

        // ===================== UI helpers =====================

        /// <summary>
        /// Animate the floating menu's width and ensure it covers the full
        /// vertical extent of the window without affecting the main layout.
        /// </summary>
        private void ToggleMenu(bool open)
        {
            LayoutOverlay();
            double targetWidth = open ? MenuExpandedWidth : 0;
            var animation = new DoubleAnimation(targetWidth, TimeSpan.FromMilliseconds(MenuAnimationDurationMs));
            MenuPanel.BeginAnimation(WidthProperty, animation);
        }


        /// <summary>
        /// Position the floating menu overlay so it covers the full window
        /// height (the width itself is animated by <see cref="ToggleMenu"/>).
        /// </summary>
        private void LayoutOverlay()
        {
            if (OverlayLayer == null || MenuPanel == null) return;
            OverlayLayer.Width = ActualWidth;
            OverlayLayer.Height = ActualHeight;
            // Keep the menu height synced with the overlay so the border paints edge-to-edge.
            MenuPanel.Height = ActualHeight;
        }

        private void ApplyTheme(bool isDark)
        {
            string themePath = isDark ? "/Resources/DarkTheme.xaml" : "/Resources/LightTheme.xaml";
            try
            {
                var themeUri = new Uri(themePath, UriKind.Relative);
                var newTheme = new ResourceDictionary { Source = themeUri };

                var mergedDicts = Application.Current.Resources.MergedDictionaries;
                for (int i = mergedDicts.Count - 1; i >= 0; i--)
                {
                    var dict = mergedDicts[i];
                    if (dict.Source != null && dict.Source.OriginalString.Contains("Theme.xaml"))
                    {
                        mergedDicts.RemoveAt(i);
                    }
                }

                mergedDicts.Add(newTheme);

                if (TxtThemeIcon != null)
                {
                    TxtThemeIcon.Text = isDark ? "☾" : "☼";
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Error al cargar el tema: {ex.Message}", "Error de Temas", MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }
    }
}
