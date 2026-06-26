/// Funciones de conversión de unidades físico-químicas.

/// Obtiene el factor de conversión de longitud respecto al Metro (m).
pub fn get_length_factor(unit: &str) -> Option<f64> {
    match unit {
        "m"  => Some(1.0),
        "km" => Some(1000.0),
        "cm" => Some(0.01),
        "mm" => Some(0.001),
        "in" => Some(0.0254),
        "ft" => Some(0.3048),
        "yd" => Some(0.9144),
        "mi" => Some(1609.344),
        _    => None,
    }
}

/// Obtiene el factor de conversión de masa respecto al Gramo (g).
pub fn get_mass_factor(unit: &str) -> Option<f64> {
    match unit {
        "g"  => Some(1.0),
        "kg" => Some(1000.0),
        "mg" => Some(0.001),
        "t"  => Some(1000000.0),
        "lb" => Some(453.59237),
        "oz" => Some(28.349523125),
        _    => None,
    }
}

/// Obtiene el factor de conversión de volumen respecto al Litro (L).
pub fn get_volume_factor(unit: &str) -> Option<f64> {
    match unit {
        "L"           => Some(1.0),
        "mL"          => Some(0.001),
        "gal"         => Some(3.785411784),
        "qt"          => Some(0.946352946),
        "pt"          => Some(0.473176473),
        "cup"         => Some(0.236588236),
        "m3" | "m³"   => Some(1000.0),
        "ft3" | "ft³" => Some(28.316846592),
        _             => None,
    }
}

/// Convierte una temperatura entre escalas (°C, °F, K).
/// Usa Celsius como escala intermedia.
pub fn convert_temp(val: f64, from: &str, to: &str) -> Option<f64> {
    // 1. Convertir a Celsius (escala base)
    let celsius = match from {
        "C" => val,
        "F" => (val - 32.0) / 1.8,
        "K" => val - 273.15,
        _   => return None,
    };
    // 2. Convertir desde Celsius al destino
    let target = match to {
        "C" => celsius,
        "F" => celsius * 1.8 + 32.0,
        "K" => celsius + 273.15,
        _   => return None,
    };
    Some(target)
}

/// Obtiene el factor de conversión de velocidad respecto al Metro por segundo (m/s).
pub fn get_velocity_factor(unit: &str) -> Option<f64> {
    match unit {
        "m/s"   => Some(1.0),
        "km/h"  => Some(1.0 / 3.6),
        "mph"   => Some(0.44704),
        "ft/s"  => Some(0.3048),
        "kn"    => Some(0.514444),  // knot (nudo)
        _       => None,
    }
}

/// Obtiene el factor de conversión de área respecto al Metro cuadrado (m²).
pub fn get_area_factor(unit: &str) -> Option<f64> {
    match unit {
        "m2" | "m²"   => Some(1.0),
        "km2" | "km²" => Some(1_000_000.0),
        "cm2" | "cm²" => Some(0.0001),
        "mm2" | "mm²" => Some(0.000001),
        "ha"           => Some(10_000.0),
        "acre"         => Some(4046.856422),
        "ft2" | "ft²"  => Some(0.092903),
        "in2" | "in²"  => Some(0.00064516),
        _              => None,
    }
}

/// Obtiene el factor de conversión de tiempo respecto al Segundo (s).
pub fn get_time_factor(unit: &str) -> Option<f64> {
    match unit {
        "s"    => Some(1.0),
        "ms"   => Some(0.001),
        "min"  => Some(60.0),
        "h"    => Some(3_600.0),
        "d"    => Some(86_400.0),
        "wk"   => Some(604_800.0),
        "mo"   => Some(2_628_000.0),  // mes promedio (365.25/12 días)
        "yr"   => Some(31_557_600.0), // año juliano (365.25 días)
        _      => None,
    }
}

/// Obtiene el factor de conversión de energía respecto al Julio (J).
pub fn get_energy_factor(unit: &str) -> Option<f64> {
    match unit {
        "J"    => Some(1.0),
        "kJ"   => Some(1_000.0),
        "cal"  => Some(4.184),
        "kcal" => Some(4_184.0),
        "Wh"   => Some(3_600.0),
        "kWh"  => Some(3_600_000.0),
        "eV"   => Some(1.602176634e-19),
        "BTU"  => Some(1055.05585),
        _      => None,
    }
}

/// Obtiene el factor de conversión de presión respecto al Pascal (Pa).
pub fn get_pressure_factor(unit: &str) -> Option<f64> {
    match unit {
        "Pa"   => Some(1.0),
        "kPa"  => Some(1_000.0),
        "MPa"  => Some(1_000_000.0),
        "bar"  => Some(100_000.0),
        "atm"  => Some(101_325.0),
        "psi"  => Some(6894.757),
        "mmHg" => Some(133.322),
        "torr" => Some(133.322),
        _      => None,
    }
}
