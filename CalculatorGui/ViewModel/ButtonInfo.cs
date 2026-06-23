using System.Windows.Input;

namespace CalculatorGui.ViewModel
{
    public class ButtonInfo : ViewModelBase
    {
        private string _normalText = "";
        private string _shiftText = "";
        private int _row;
        private int _column;
        private string _styleKey = "SciButtonStyle";
        private string _specialType = "";
        private ICommand? _command;
        private bool _isActive;

        public string NormalText
        {
            get => _normalText;
            set => SetProperty(ref _normalText, value);
        }

        public string ShiftText
        {
            get => _shiftText;
            set => SetProperty(ref _shiftText, value);
        }

        public int Row
        {
            get => _row;
            set => SetProperty(ref _row, value);
        }

        public int Column
        {
            get => _column;
            set => SetProperty(ref _column, value);
        }

        public string StyleKey
        {
            get => _styleKey;
            set => SetProperty(ref _styleKey, value);
        }

        public string SpecialType
        {
            get => _specialType;
            set => SetProperty(ref _specialType, value);
        }

        public ICommand? Command
        {
            get => _command;
            set => SetProperty(ref _command, value);
        }

        public bool IsActive
        {
            get => _isActive;
            set => SetProperty(ref _isActive, value);
        }
    }
}
