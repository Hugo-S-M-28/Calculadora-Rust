using System.Collections.ObjectModel;
using System.Globalization;

namespace CalculatorGui.ViewModel
{
    public class UnitItem
    {
        public string Tag { get; set; } = "";
        public string Name { get; set; } = "";
    }

    public class CategoryItem
    {
        public string Tag { get; set; } = "";
        public string Name { get; set; } = "";
    }

    public class ConverterViewModel : ViewModelBase
    {
        private string _category = "";
        private string _fromValue = "1";
        private string _toValue = "0";
        private UnitItem? _selectedFromUnit;
        private UnitItem? _selectedToUnit;
        private CategoryItem? _selectedCategory;

        public ObservableCollection<UnitItem> Units { get; } = new ObservableCollection<UnitItem>();
        public ObservableCollection<CategoryItem> Categories { get; } = new ObservableCollection<CategoryItem>();

        public ConverterViewModel()
        {
            Categories.Add(new CategoryItem { Tag = "length", Name = "Longitud" });
            Categories.Add(new CategoryItem { Tag = "temperature", Name = "Temperatura" });
            Categories.Add(new CategoryItem { Tag = "mass", Name = "Masa" });
            Categories.Add(new CategoryItem { Tag = "volume", Name = "Volumen" });
            SelectedCategory = Categories[0];
        }

        public string Category
        {
            get => _category;
            set
            {
                if (SetProperty(ref _category, value))
                {
                    PopulateUnits();
                    if (SelectedCategory == null || SelectedCategory.Tag != value)
                    {
                        foreach (var cat in Categories)
                        {
                            if (cat.Tag == value)
                            {
                                SelectedCategory = cat;
                                break;
                            }
                        }
                    }
                }
            }
        }

        public CategoryItem? SelectedCategory
        {
            get => _selectedCategory;
            set
            {
                if (SetProperty(ref _selectedCategory, value) && value != null)
                {
                    Category = value.Tag;
                }
            }
        }

        public string FromValue
        {
            get => _fromValue;
            set
            {
                if (SetProperty(ref _fromValue, value))
                {
                    PerformConversion();
                }
            }
        }

        public string ToValue
        {
            get => _toValue;
            private set => SetProperty(ref _toValue, value);
        }

        public UnitItem? SelectedFromUnit
        {
            get => _selectedFromUnit;
            set
            {
                if (SetProperty(ref _selectedFromUnit, value))
                {
                    PerformConversion();
                }
            }
        }

        public UnitItem? SelectedToUnit
        {
            get => _selectedToUnit;
            set
            {
                if (SetProperty(ref _selectedToUnit, value))
                {
                    PerformConversion();
                }
            }
        }

        private void PopulateUnits()
        {
            Units.Clear();
            if (Category == "length")
            {
                Units.Add(new UnitItem { Tag = "m", Name = "Metros (m)" });
                Units.Add(new UnitItem { Tag = "km", Name = "Kilómetros (km)" });
                Units.Add(new UnitItem { Tag = "cm", Name = "Centímetros (cm)" });
                Units.Add(new UnitItem { Tag = "mm", Name = "Milímetros (mm)" });
                Units.Add(new UnitItem { Tag = "in", Name = "Pulgadas (in)" });
                Units.Add(new UnitItem { Tag = "ft", Name = "Pies (ft)" });
                Units.Add(new UnitItem { Tag = "yd", Name = "Yardas (yd)" });
                Units.Add(new UnitItem { Tag = "mi", Name = "Millas (mi)" });
            }
            else if (Category == "temperature")
            {
                Units.Add(new UnitItem { Tag = "C", Name = "Celsius (°C)" });
                Units.Add(new UnitItem { Tag = "F", Name = "Fahrenheit (°F)" });
                Units.Add(new UnitItem { Tag = "K", Name = "Kelvin (K)" });
            }
            else if (Category == "mass")
            {
                Units.Add(new UnitItem { Tag = "g", Name = "Gramos (g)" });
                Units.Add(new UnitItem { Tag = "kg", Name = "Kilogramos (kg)" });
                Units.Add(new UnitItem { Tag = "mg", Name = "Miligramos (mg)" });
                Units.Add(new UnitItem { Tag = "t", Name = "Toneladas (t)" });
                Units.Add(new UnitItem { Tag = "lb", Name = "Libras (lb)" });
                Units.Add(new UnitItem { Tag = "oz", Name = "Onzas (oz)" });
            }
            else if (Category == "volume")
            {
                Units.Add(new UnitItem { Tag = "L", Name = "Litros (L)" });
                Units.Add(new UnitItem { Tag = "mL", Name = "Mililitros (mL)" });
                Units.Add(new UnitItem { Tag = "gal", Name = "Galones (gal)" });
                Units.Add(new UnitItem { Tag = "qt", Name = "Cuartos (qt)" });
                Units.Add(new UnitItem { Tag = "pt", Name = "Pintas (pt)" });
                Units.Add(new UnitItem { Tag = "cup", Name = "Tazas (cup)" });
                Units.Add(new UnitItem { Tag = "m3", Name = "Metros Cúbicos (m³)" });
                Units.Add(new UnitItem { Tag = "ft3", Name = "Pies Cúbicos (ft³)" });
            }

            if (Units.Count > 0) SelectedFromUnit = Units[0];
            if (Units.Count > 1) SelectedToUnit = Units[1];
            else if (Units.Count > 0) SelectedToUnit = Units[0];

            PerformConversion();
        }

        private void PerformConversion()
        {
            if (SelectedFromUnit == null || SelectedToUnit == null) return;

            string valStr = FromValue.Trim().Replace(',', '.');
            if (double.TryParse(valStr, NumberStyles.Any, CultureInfo.InvariantCulture, out double value))
            {
                double result = CalculatorBridge.ConvertUnits(value, SelectedFromUnit.Tag, SelectedToUnit.Tag, Category);
                ToValue = result.ToString("G9", CultureInfo.InvariantCulture);
            }
            else
            {
                ToValue = string.IsNullOrEmpty(valStr) ? "0" : "Error";
            }
        }
    }
}
