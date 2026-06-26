/// suite_regression.rs — Pruebas de regresión de bugs (~25 casos)
///
/// Cada test corresponde a un bug específico identificado en el análisis.
/// Si alguno falla, significa que el bug ha reaparecido (regresión).

use crate::calculator::calculator::{
    process_expression, set_angle_mode, set_decimal_precision,
};
use crate::calculator::value::clear_variables;
use std::sync::Mutex;
use crate::calculator::TEST_MUTEX as MX;

/// Recupera el MutexGuard incluso si está envenenado por un panic previo.
fn lock_silent(m: &Mutex<()>) -> std::sync::MutexGuard<'_, ()> {
    match m.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// BUG-1: Las letras c, g, h ya NO son constantes físicas
//        (ahora son c_light, G_grav, h_planck)
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_bug1_c_is_free() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    clear_variables();
    // Asignar c como variable — no debe ser la velocidad de la luz
    process_expression("c = 5").unwrap();
    let r: f64 = process_expression("c").unwrap().parse().unwrap();
    assert_eq!(r, 5.0, "BUG-1: c debe ser variable libre, no constante física");
}

#[test]
fn regression_bug1_g_is_free() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    clear_variables();
    process_expression("g = 9").unwrap();
    let r: f64 = process_expression("g").unwrap().parse().unwrap();
    assert_eq!(r, 9.0, "BUG-1: g debe ser variable libre, no 9.81");
}

#[test]
fn regression_bug1_h_is_free() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    clear_variables();
    process_expression("h = 7").unwrap();
    let r: f64 = process_expression("h").unwrap().parse().unwrap();
    assert_eq!(r, 7.0, "BUG-1: h debe ser variable libre, no h_planck");
}

#[test]
fn regression_bug1_renamed_constants() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // Las constantes renombradas deben seguir funcionando
    let c_light: f64 = process_expression("c_light").unwrap().parse().unwrap();
    assert_eq!(c_light, 299_792_458.0, "c_light debe ser 299792458");
    assert!(process_expression("G_grav").is_ok(), "G_grav debe existir");
    assert!(process_expression("h_planck").is_ok(), "h_planck debe existir");
}

// ─────────────────────────────────────────────────────────────────────────
// BUG-2: No perder el carácter 'p' en handle_number_constant_combination
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_bug2_3p_not_silent_fail() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    clear_variables();
    // "3p" sin definir p → debe fallar (no silenciosamente = 0)
    let r = process_expression("3p");
    assert!(r.is_err(), "BUG-2: '3p' con p indefinida debe ser error, no 0");
}

#[test]
fn regression_bug2_2pi_correct() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // "2pi" debe ser 2×π, no "2p" × "i"
    let r: f64 = process_expression("2pi").unwrap().parse().unwrap();
    assert!((r - 2.0 * std::f64::consts::PI).abs() < 1e-7,
        "BUG-2: 2pi debe ser 2π ≈ 6.283, got={}", r);
}

// ─────────────────────────────────────────────────────────────────────────
// BUG-3: Ambigüedad entre constante 'e' y notación científica
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_bug3_scientific_notation_priority() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // 1.2e3 debe ser 1200 (notación científica), no 1.2 × e × 3
    let r: f64 = process_expression("1.2e3").unwrap().parse().unwrap();
    assert_eq!(r, 1200.0, "BUG-3: 1.2e3 debe ser 1200");
}

#[test]
fn regression_bug3_1e6() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    let r: f64 = process_expression("1e6").unwrap().parse().unwrap();
    assert_eq!(r, 1_000_000.0, "BUG-3: 1e6 debe ser 1000000");
}

#[test]
fn regression_bug3_3e_is_product() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // "3e" sin dígitos de exponente → 3 × e
    let r: f64 = process_expression("3e").unwrap().parse().unwrap();
    assert!((r - 3.0 * std::f64::consts::E).abs() < 1e-6,
        "BUG-3: 3e debe ser 3×e ≈ 8.154, got={}", r);
}

// ─────────────────────────────────────────────────────────────────────────
// BUG-4 / BUG-5: Número mal formado con dos puntos decimales
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_bug4_double_dot_is_error() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    assert!(process_expression("1.2.3").is_err(),
        "BUG-4/5: '1.2.3' debe ser SyntaxError, no 0.0");
}

#[test]
fn regression_bug5_consecutive_dots() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    assert!(process_expression("1..2").is_err(),
        "BUG-5: '1..2' debe ser SyntaxError");
}

// ─────────────────────────────────────────────────────────────────────────
// BUG-7: convert_units_ffi usa f64::NAN en error, no 0.0
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_bug7_unknown_unit_is_nan() {
    // Verificar a nivel de units.rs que unidad desconocida → None
    let factor = crate::units::get_length_factor("lightyear");
    assert!(factor.is_none(), "BUG-7: unidad desconocida debe ser None, no 0");
}

#[test]
fn regression_bug7_unknown_category_nan() {
    // category desconocida → NAN
    let result = crate::units::convert_temp(100.0, "X", "Y");
    assert!(result.is_none(), "BUG-7: categoría/unidad inválida debe ser None");
}

// ─────────────────────────────────────────────────────────────────────────
// BUG-8: preprocess_tokens O(1) — no pierde paréntesis al auto-cerrar
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_bug8_autoclose_correct() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // "(1 + 2" auto-cierra a "(1+2)" = 3, no a "(1" = error
    let r: f64 = process_expression("(1 + 2").unwrap().parse().unwrap();
    assert_eq!(r, 3.0, "BUG-8: auto-cierre debe dar 3");
}

#[test]
fn regression_bug8_deep_autoclose() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // "((5" → auto-cierra doble → 5
    let r: f64 = process_expression("((5").unwrap().parse().unwrap();
    assert_eq!(r, 5.0, "BUG-8: doble auto-cierre debe dar 5");
}

// ─────────────────────────────────────────────────────────────────────────
// BUG-9: Precedencia de unario '-' vs '^'
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_bug9_neg_before_pow() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // -2^2 = -(2^2) = -4, NO (-2)^2 = 4
    let r: f64 = process_expression("-2^2").unwrap().parse().unwrap();
    assert_eq!(r, -4.0, "BUG-9: -2^2 debe ser -4, no 4");
}

#[test]
fn regression_bug9_neg_paren_pow() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // (-2)^2 = 4
    let r: f64 = process_expression("(-2)^2").unwrap().parse().unwrap();
    assert_eq!(r, 4.0, "BUG-9: (-2)^2 debe ser 4");
}

#[test]
fn regression_bug9_neg_neg_exp() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // -2^-2 = -(2^-2) = -0.25
    let r: f64 = process_expression("-2^-2").unwrap().parse().unwrap();
    assert_eq!(r, -0.25, "BUG-9: -2^-2 debe ser -0.25");
}

// ─────────────────────────────────────────────────────────────────────────
// BUG-10: Funciones trigonométricas con sufijo 'e' o 'pi'
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_bug10_sine() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // "sine" → sin(e) ≈ 0.41078129
    let r: f64 = process_expression("sine").unwrap().parse().unwrap();
    assert!((r - (std::f64::consts::E).sin()).abs() < 1e-7,
        "BUG-10: 'sine' debe ser sin(e), got={}", r);
}

#[test]
fn regression_bug10_cose() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    let r: f64 = process_expression("cose").unwrap().parse().unwrap();
    assert!((r - (std::f64::consts::E).cos()).abs() < 1e-7,
        "BUG-10: 'cose' debe ser cos(e), got={}", r);
}

#[test]
fn regression_bug10_tane() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    let r: f64 = process_expression("tane").unwrap().parse().unwrap();
    assert!((r - (std::f64::consts::E).tan()).abs() < 1e-7,
        "BUG-10: 'tane' debe ser tan(e), got={}", r);
}

#[test]
fn regression_bug10_sinpi() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    let r: f64 = process_expression("sinpi").unwrap().parse().unwrap();
    assert!(r.abs() < 1e-7, "BUG-10: 'sinpi' debe ser sin(π) ≈ 0, got={}", r);
}

#[test]
fn regression_bug10_cospi() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    let r: f64 = process_expression("cospi").unwrap().parse().unwrap();
    assert_eq!(r, -1.0, "BUG-10: 'cospi' debe ser cos(π) = -1");
}

// ─────────────────────────────────────────────────────────────────────────
// BUG-11: GCD/LCM overflow silencioso
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_bug11_gcd_overflow() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // gcd(1e20, 2) debe fallar, no silenciosamente devolver resultado incorrecto
    let r = process_expression("gcd(1e20, 2)");
    assert!(r.is_err(), "BUG-11: gcd(1e20, 2) debe ser error por overflow");
}

#[test]
fn regression_bug11_gcd_float_consistent() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    // gcd(12.5, 5) — si devuelve valor, debe ser finito (no NaN, no panic)
    let r = process_expression("gcd(12.5, 5)");
    if let Ok(v) = r {
        let n: f64 = v.parse().unwrap_or(f64::NAN);
        assert!(n.is_finite(), "BUG-11: gcd(12.5,5) no debe ser NaN");
    }
}

// ─────────────────────────────────────────────────────────────────────────
// MEJ-5: nCr/nPr iterativo — no overflow para n=100
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_mej5_ncr_100_50_finite() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    let r: f64 = process_expression("nCr(100, 50)").unwrap().parse().unwrap();
    assert!(r.is_finite() && r > 0.0,
        "MEJ-5: nCr(100,50) debe ser finito y positivo, got={}", r);
}

#[test]
fn regression_mej5_npr_20_10_finite() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    let r: f64 = process_expression("nPr(20, 10)").unwrap().parse().unwrap();
    assert!(r.is_finite() && r > 0.0,
        "MEJ-5: nPr(20,10) debe ser finito y positivo, got={}", r);
}

// ─────────────────────────────────────────────────────────────────────────
// MEJ-11: ** como alias de ^
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_mej11_power_alias() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    let a: f64 = process_expression("2**10").unwrap().parse().unwrap();
    let b: f64 = process_expression("2^10").unwrap().parse().unwrap();
    assert_eq!(a, b, "MEJ-11: 2**10 debe ser igual que 2^10");
    assert_eq!(a, 1024.0);
}

// ─────────────────────────────────────────────────────────────────────────
// MEJ-10: Precisión decimal configurable
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_mej10_precision_changes() {
    let _g = lock_silent(&MX); set_angle_mode(0);
    use crate::calculator::calculator::{set_decimal_precision, round_to_precision};
    set_decimal_precision(4);
    let v = round_to_precision(1.0 / 3.0);
    assert_eq!(v, 0.3333, "MEJ-10: 4 dec deben dar 0.3333");
    set_decimal_precision(8); // restaurar
}

// ─────────────────────────────────────────────────────────────────────────
// MEJ-8: 3 modos de ángulo (RAD/DEG/GRAD)
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_mej8_three_modes() {
    let _g = lock_silent(&MX); set_decimal_precision(8);
    // RAD
    set_angle_mode(0);
    let sin_rad: f64 = process_expression("sin(pi/2)").unwrap().parse().unwrap();
    assert_eq!(sin_rad, 1.0, "RAD: sin(π/2)=1");
    // DEG
    set_angle_mode(1);
    let sin_deg: f64 = process_expression("sin(90)").unwrap().parse().unwrap();
    assert_eq!(sin_deg, 1.0, "DEG: sin(90)=1");
    // GRAD
    set_angle_mode(2);
    let sin_grad: f64 = process_expression("sin(100)").unwrap().parse().unwrap();
    assert_eq!(sin_grad, 1.0, "GRAD: sin(100)=1");
    set_angle_mode(0);
}

// ─────────────────────────────────────────────────────────────────────────
// MEJ-14: Nuevas categorías de conversión de unidades
// ─────────────────────────────────────────────────────────────────────────
#[test]
fn regression_mej14_velocity() {
    let f_ms  = crate::units::get_velocity_factor("m/s").unwrap();
    let f_kmh = crate::units::get_velocity_factor("km/h").unwrap();
    let r = (1.0 * f_ms) / f_kmh;
    assert!((r - 3.6).abs() < 1e-6, "MEJ-14: 1 m/s → 3.6 km/h, got={}", r);
}

#[test]
fn regression_mej14_energy() {
    let f_kcal = crate::units::get_energy_factor("kcal").unwrap();
    let f_j    = crate::units::get_energy_factor("J").unwrap();
    let r = (1.0 * f_kcal) / f_j;
    assert!((r - 4184.0).abs() < 0.1, "MEJ-14: 1 kcal → 4184 J, got={}", r);
}

#[test]
fn regression_mej14_pressure() {
    let f_atm = crate::units::get_pressure_factor("atm").unwrap();
    let f_pa  = crate::units::get_pressure_factor("Pa").unwrap();
    let r = (1.0 * f_atm) / f_pa;
    assert!((r - 101325.0).abs() < 0.1, "MEJ-14: 1 atm → 101325 Pa, got={}", r);
}

#[test]
fn regression_new_sum() {
    let _g = lock_silent(&MX);
    let r = process_expression("sum([2, 4, 6])").unwrap();
    assert_eq!(r, "12");
}

#[test]
fn regression_new_min() {
    let _g = lock_silent(&MX);
    let r = process_expression("min([10, -5, 20])").unwrap();
    assert_eq!(r, "-5");
}

#[test]
fn regression_new_max() {
    let _g = lock_silent(&MX);
    let r = process_expression("max([10, -5, 20])").unwrap();
    assert_eq!(r, "20");
}

#[test]
fn regression_new_sort() {
    let _g = lock_silent(&MX);
    let r = process_expression("sort([9, 2, 5])").unwrap();
    assert_eq!(r, "[2, 5, 9]");
}

#[test]
fn regression_new_tr() {
    let _g = lock_silent(&MX);
    let r = process_expression("tr([5, 1; 2, 10])").unwrap();
    assert_eq!(r, "15");
}

#[test]
fn regression_new_log_base() {
    let _g = lock_silent(&MX);
    let r = process_expression("log(16, 2)").unwrap();
    assert_eq!(r, "4");
}

#[test]
fn regression_new_sqrt_complex() {
    let _g = lock_silent(&MX);
    let r = process_expression("sqrt(-16)").unwrap();
    assert_eq!(r, "4i");
}

#[test]
fn regression_new_complex_spaces() {
    let _g = lock_silent(&MX);
    let r = process_expression("(1 + 2i) * 2").unwrap();
    assert_eq!(r, "2+4i");
}

#[test]
fn regression_new_factorial_inf() {
    let _g = lock_silent(&MX);
    let r = process_expression("171!").unwrap();
    assert_eq!(r, "Infinity");
}

#[test]
fn regression_percent_binary() {
    let _g = lock_silent(&MX);
    let r = process_expression("10 % 200").unwrap();
    assert_eq!(r, "20");
}
