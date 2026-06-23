using System.Collections.ObjectModel;
using System.Windows.Input;
using CalculatorGui;

namespace CalculatorGui.ViewModel
{
    public class MainViewModel : ViewModelBase
    {
        private ViewModelBase _currentViewModel;
        private string _appTitle = "Calculadora - Científica";
        private bool _isMenuOpen = false;
        private bool _isHistoryOpen = true;

        public ObservableCollection<HistoryItem> History { get; } = new ObservableCollection<HistoryItem>();

        // Precreated subview ViewModels to keep their state when navigating
        public CalculatorViewModel CalculatorVm { get; }
        public ConverterViewModel ConverterVm { get; }
        public GraphingViewModel GraphingVm { get; }

        public MainViewModel()
        {
            CalculatorVm = new CalculatorViewModel(History);
            ConverterVm = new ConverterViewModel();
            GraphingVm = new GraphingViewModel();

            // Default to Scientific Calculator
            CalculatorVm.IsScientific = true;
            _currentViewModel = CalculatorVm;

            // Commands
            NavigateCommand = new RelayCommand(param => Navigate(param as string));
            ToggleMenuCommand = new RelayCommand(_ => IsMenuOpen = !IsMenuOpen);
            ToggleHistoryCommand = new RelayCommand(_ => IsHistoryOpen = !IsHistoryOpen);
        }

        public ViewModelBase CurrentViewModel
        {
            get => _currentViewModel;
            set => SetProperty(ref _currentViewModel, value);
        }

        public string AppTitle
        {
            get => _appTitle;
            set => SetProperty(ref _appTitle, value);
        }

        public bool IsMenuOpen
        {
            get => _isMenuOpen;
            set => SetProperty(ref _isMenuOpen, value);
        }

        public bool IsHistoryOpen
        {
            get => _isHistoryOpen;
            set => SetProperty(ref _isHistoryOpen, value);
        }

        public ICommand NavigateCommand { get; }
        public ICommand ToggleMenuCommand { get; }
        public ICommand ToggleHistoryCommand { get; }

        private void Navigate(string? destination)
        {
            if (string.IsNullOrEmpty(destination)) return;

            switch (destination)
            {
                case "Standard":
                    AppTitle = "Calculadora - Estándar";
                    CalculatorVm.IsScientific = false;
                    CurrentViewModel = CalculatorVm;
                    break;
                case "Scientific":
                    AppTitle = "Calculadora - Científica";
                    CalculatorVm.IsScientific = true;
                    CurrentViewModel = CalculatorVm;
                    break;
                case "Length":
                    AppTitle = "Calculadora - Conversor de Longitud";
                    ConverterVm.Category = "length";
                    CurrentViewModel = ConverterVm;
                    break;
                case "Temperature":
                    AppTitle = "Calculadora - Conversor de Temperatura";
                    ConverterVm.Category = "temperature";
                    CurrentViewModel = ConverterVm;
                    break;
                case "Converter":
                    AppTitle = "Calculadora - Conversor de Unidades";
                    CurrentViewModel = ConverterVm;
                    break;
                case "Graphing":
                    AppTitle = "Calculadora - Graficador de Funciones";
                    CurrentViewModel = GraphingVm;
                    break;
            }

            // Close navigation menu drawer after selection
            IsMenuOpen = false;
        }
    }
}
