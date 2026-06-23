namespace CalculatorGui.ViewModel
{
    public class GraphingViewModel : ViewModelBase
    {
        private string _functionExpression = "sin(x)";

        public string FunctionExpression
        {
            get => _functionExpression;
            set => SetProperty(ref _functionExpression, value);
        }
    }
}
