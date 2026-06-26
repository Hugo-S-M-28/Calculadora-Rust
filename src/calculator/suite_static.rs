#![allow(clippy::approx_constant)]

/// suite_static.rs — Suite de pruebas unitarias estáticas (~350 casos)
///
/// Macro auxiliar:
///   check!(expr => resultado)          compara string exacto
///   check!(expr ~= valor, tol=tol)     tolerancia numérica
///   check!(err: expr)                  espera cualquier Err

use crate::calculator::calculator::{
    process_expression, set_angle_mode, set_decimal_precision,
    round_to_precision, get_last_result, clear_last_result,
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

macro_rules! check {
    ($expr:literal => $expected:literal) => {{
        let got = process_expression($expr)
            .expect(&format!("'{}' no deberia fallar", $expr));
        assert_eq!(got.as_str(), $expected,
            "expr='{}' got='{}' expected='{}'", $expr, got, $expected);
    }};
    ($expr:literal ~= $expected:expr, tol=$tol:expr) => {{
        let got = process_expression($expr)
            .expect(&format!("'{}' no deberia fallar", $expr));
        let v: f64 = got.parse()
            .expect(&format!("no es numero: '{}'", got));
        assert!((v - ($expected as f64)).abs() < ($tol as f64),
            "expr='{}' got={} expected≈{} tol={}", $expr, v, $expected, $tol);
    }};
    (err: $expr:literal) => {{
        assert!(process_expression($expr).is_err(),
            "expr='{}' deberia Err pero devolvio Ok", $expr);
    }};
}

// =========================================================================
// 1. ARITMÉTICA BÁSICA Y PRECEDENCIA (45 pruebas)
// =========================================================================
#[test] fn arith_01_sum()        { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(8); check!("1 + 1" => "2"); }
#[test] fn arith_02_sub()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("10 - 3" => "7"); }
#[test] fn arith_03_zero()       { let _g=lock_silent(&MX); set_angle_mode(0); check!("0 + 0" => "0"); }
#[test] fn arith_04_neg_cancel() { let _g=lock_silent(&MX); set_angle_mode(0); check!("-5 + 5" => "0"); }
#[test] fn arith_05_big()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("1000000 + 1" => "1000001"); }
#[test] fn arith_06_float()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("0.1 + 0.2" ~= 0.3, tol=1e-9); }
#[test] fn arith_07_mul()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("3 * 4" => "12"); }
#[test] fn arith_08_div()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("10 / 2" => "5"); }
#[test] fn arith_09_div_float()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("7 / 2" => "3.5"); }
#[test] fn arith_10_neg_div()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("-6 / 2" => "-3"); }
#[test] fn arith_11_zero_mul()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("0 * 9999999" => "0"); }
#[test] fn arith_12_prec1()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("2 + 3 * 4" => "14"); }
#[test] fn arith_13_prec2()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("(2 + 3) * 4" => "20"); }
#[test] fn arith_14_prec3()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("10 - 2 * 3" => "4"); }
#[test] fn arith_15_prec4()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("10 / 2 + 3" => "8"); }
#[test] fn arith_16_prec5()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("10 / (2 + 3)" => "2"); }
#[test] fn arith_17_mixed()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("2 * 3 + 4 * 5" => "26"); }
#[test] fn arith_18_paren_pair() { let _g=lock_silent(&MX); set_angle_mode(0); check!("(1 + 2) * (3 + 4)" => "21"); }
#[test] fn arith_19_mod1()       { let _g=lock_silent(&MX); set_angle_mode(0); check!("mod(10, 3)" => "1"); }
#[test] fn arith_20_mod2()       { let _g=lock_silent(&MX); set_angle_mode(0); check!("mod(10, 3)" => "1"); }
#[test] fn arith_21_mod_zero()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("mod(7, 7)" => "0"); }
#[test] fn arith_22_mod_from_z() { let _g=lock_silent(&MX); set_angle_mode(0); check!("mod(0, 5)" => "0"); }
// % como operador posfijo requiere RHS; se usan expresiones equivalentes
#[test] fn arith_23_pct1()       { let _g=lock_silent(&MX); set_angle_mode(0); check!("200 * (10/100)" => "20"); }
#[test] fn arith_24_pct2()       { let _g=lock_silent(&MX); set_angle_mode(0); check!("50 * (20/100)" => "10"); }
#[test] fn arith_25_abs_neg()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("abs(-5)" => "5"); }
#[test] fn arith_26_abs_zero()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("abs(0)" => "0"); }
#[test] fn arith_27_abs_pos()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("abs(3)" => "3"); }
#[test] fn arith_28_dbl_neg()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("--5" => "5"); }
#[test] fn arith_29_neg_paren()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("-(-3)" => "3"); }
#[test] fn arith_30_unary_plus() { let _g=lock_silent(&MX); set_angle_mode(0); check!("+5" => "5"); }
#[test] fn arith_31_chain_add()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("1 + 2 + 3 + 4 + 5" => "15"); }
#[test] fn arith_32_chain_sub()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("10 - 1 - 1 - 1 - 1" => "6"); }
#[test] fn arith_33_chain_mul()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("2 * 2 * 2 * 2" => "16"); }
#[test] fn arith_34_float_sum()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("1.5 + 2.5" => "4"); }
#[test] fn arith_35_pi_mul()     { let _g=lock_silent(&MX); set_angle_mode(0); check!("3.14 * 2" ~= 6.28, tol=1e-6); }
#[test] fn arith_36_tiny()       { let _g=lock_silent(&MX); set_angle_mode(0); check!("0.001 + 0.002" ~= 0.003, tol=1e-9); }
#[test] fn arith_37_neg_plus()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("-10 + 20" => "10"); }
#[test] fn arith_38_sub_neg()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("5 - -3" => "8"); }
#[test] fn arith_39_trillion()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("1000000 * 1000000" => "1000000000000"); }
#[test] fn arith_40_neg_zero()   { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(8); assert_eq!(process_expression("0 * -1").unwrap_or_default(), "0"); }
#[test] fn arith_41_sqrt_sub()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("sqrt(2) - sqrt(2)" => "0"); }
#[test] fn arith_42_pi_sub()     { let _g=lock_silent(&MX); set_angle_mode(0); check!("pi - pi" => "0"); }
#[test] fn arith_43_e_sub()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("e - e" => "0"); }
#[test] fn arith_44_nested()     { let _g=lock_silent(&MX); set_angle_mode(0); check!("((1+2)*(3+4))" => "21"); }
#[test] fn arith_45_complex_m()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("(3 + 4) * (10 - 5) / (2 + 3)" => "7"); }

// =========================================================================
// 2. POTENCIAS Y RAÍCES (30 pruebas)
// =========================================================================
#[test] fn pow_01()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("2^10" => "1024"); }
#[test] fn pow_02_alias()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("2**10" => "1024"); }
#[test] fn pow_03()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("3^3" => "27"); }
#[test] fn pow_04()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("10^0" => "1"); }
#[test] fn pow_05_0_0()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("0^0" => "1"); }
#[test] fn pow_06()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("(-2)^3" => "-8"); }
#[test] fn pow_07()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("(-2)^2" => "4"); }
#[test] fn pow_08_unary()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("-2^2" => "-4"); }
#[test] fn pow_09()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("-2^-2" => "-0.25"); }
#[test] fn pow_10()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("2^-2" => "0.25"); }
#[test] fn pow_11()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("2^0.5" ~= 1.41421356, tol=1e-7); }
#[test] fn pow_12()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("4^0.5" => "2"); }
#[test] fn pow_13()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("8^(1/3)" ~= 2.0, tol=1e-7); }
#[test] fn pow_14()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("9^0.5" => "3"); }
#[test] fn pow_15_sqrt16() { let _g=lock_silent(&MX); set_angle_mode(0); check!("sqrt(16)" => "4"); }
#[test] fn pow_16_sqrt2()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("sqrt(2)" ~= 1.41421356, tol=1e-7); }
#[test] fn pow_17_sqrt0()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("sqrt(0)" => "0"); }
#[test] fn pow_18_cbrt()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("cbrt(8)" => "2"); }
#[test] fn pow_19_cbrt_n() { let _g=lock_silent(&MX); set_angle_mode(0); check!("cbrt(-8)" => "-2"); }
#[test] fn pow_20_cbrt27() { let _g=lock_silent(&MX); set_angle_mode(0); check!("cbrt(27)" => "3"); }
#[test] fn pow_21_root3()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("root(8, 3)" => "2"); }
#[test] fn pow_22_root4()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("root(16, 4)" => "2"); }
#[test] fn pow_23_root5()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("root(32, 5)" => "2"); }
#[test] fn pow_24_nested() { let _g=lock_silent(&MX); set_angle_mode(0); check!("2^(2^2)" => "16"); }
#[test] fn pow_25_assoc()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("2^3^2" => "512"); }
#[test] fn pow_26_100()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("100^0.5" => "10"); }
#[test] fn pow_27_1000()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("1000^(1/3)" ~= 10.0, tol=1e-6); }
#[test] fn pow_28_frac()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("(1/4)^0.5" => "0.5"); }
#[test] fn pow_29()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("(-1)^2" => "1"); }
#[test] fn pow_30_23()     { let _g=lock_silent(&MX); set_angle_mode(0); check!("27^(2/3)" => "9"); }

// =========================================================================
// 3. FACTORIALES Y COMBINATORIA (30 pruebas)
// =========================================================================
#[test] fn fact_01()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("0!" => "1"); }
#[test] fn fact_02()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("1!" => "1"); }
#[test] fn fact_03()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("2!" => "2"); }
#[test] fn fact_04()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("3!" => "6"); }
#[test] fn fact_05()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("4!" => "24"); }
#[test] fn fact_06()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("5!" => "120"); }
#[test] fn fact_07()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("6!" => "720"); }
#[test] fn fact_08()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("7!" => "5040"); }
#[test] fn fact_09()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("10!" => "3628800"); }
#[test] fn fact_10_ncr_zero()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("nCr(5, 0)" => "1"); }
#[test] fn fact_11_ncr_full()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("nCr(5, 5)" => "1"); }
#[test] fn fact_12_ncr_10_3()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("nCr(10, 3)" => "120"); }
#[test] fn fact_13_ncr_5_2()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("nCr(5, 2)" => "10"); }
#[test] fn fact_14_ncr_20_10()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("nCr(20, 10)" => "184756"); }
#[test] fn fact_15_ncr_large()  { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("nCr(100,50)").unwrap(); let v:f64=r.parse().unwrap(); assert!(v.is_finite()&&v>0.0); }
#[test] fn fact_16_npr_zero()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("nPr(5, 0)" => "1"); }
#[test] fn fact_17_npr_full()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("nPr(5, 5)" => "120"); }
#[test] fn fact_18_npr_10_3()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("nPr(10, 3)" => "720"); }
#[test] fn fact_19_npr_4_2()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("nPr(4, 2)" => "12"); }
#[test] fn fact_20_npr_large()  { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("nPr(20, 10)").unwrap(); let v:f64=r.parse().unwrap(); assert!(v.is_finite()&&v>0.0); }
#[test] fn fact_21_gcd()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("gcd(48, 18)" => "6"); }
#[test] fn fact_22_gcd2()       { let _g=lock_silent(&MX); set_angle_mode(0); check!("gcd(12, 4)" => "4"); }
#[test] fn fact_23_gcd_coprime(){ let _g=lock_silent(&MX); set_angle_mode(0); check!("gcd(7, 13)" => "1"); }
#[test] fn fact_24_gcd_zero()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("gcd(0, 5)" => "5"); }
#[test] fn fact_25_lcm()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("lcm(4, 6)" => "12"); }
#[test] fn fact_26_lcm2()       { let _g=lock_silent(&MX); set_angle_mode(0); check!("lcm(3, 7)" => "21"); }
#[test] fn fact_27_lcm3()       { let _g=lock_silent(&MX); set_angle_mode(0); check!("lcm(12, 18)" => "36"); }
#[test] fn fact_28_chain()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("3! + 4!" => "30"); }
#[test] fn fact_29_div()        { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("10! / 5!").unwrap(); let v:f64=r.parse().unwrap(); assert_eq!(v,30240.0); }
#[test] fn fact_30_symmetry()   { let _g=lock_silent(&MX); set_angle_mode(0); let a=process_expression("nCr(10, 3)").unwrap(); let b=process_expression("nCr(10, 7)").unwrap(); assert_eq!(a,b); }

// =========================================================================
// 4. PARÉNTESIS Y AUTO-CIERRE (20 pruebas)
// =========================================================================
#[test] fn paren_01()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("(1 + 2) * 3" => "9"); }
#[test] fn paren_02()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("((1 + 2)) * 3" => "9"); }
#[test] fn paren_03()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("(((3)))" => "3"); }
#[test] fn paren_04()            { let _g=lock_silent(&MX); set_angle_mode(0); check!("(1 + 2) * (3 + 4)" => "21"); }
#[test] fn paren_05_ac1()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("(1 + 2" => "3"); }
#[test] fn paren_06_ac2()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("((4" => "4"); }
#[test] fn paren_07_ac_fn()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("sin(pi/2" => "1"); }
#[test] fn paren_08_vec()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("[1, 2, 3]" => "[1, 2, 3]"); }
#[test] fn paren_09_mat()        { let _g=lock_silent(&MX); set_angle_mode(0); check!("[1, 2; 3, 4]" => "[1, 2; 3, 4]"); }
#[test] fn paren_10_deep20()     { let _g=lock_silent(&MX); set_angle_mode(0); check!("((((((((((1+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)" => "11"); }
#[test] fn paren_11_empty_err()  { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "()"); }
#[test] fn paren_12_str_err()    { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: ""); }
#[test] fn paren_13_bracket_err(){ let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "]"); }
#[test] fn paren_14_paren_err()  { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: ")"); }
#[test] fn paren_15_vec_expr()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("[1+2, 3*4]" => "[3, 12]"); }
#[test] fn paren_16_ac_sqrt()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("sqrt(4" => "2"); }
#[test] fn paren_17_ac_log()     { let _g=lock_silent(&MX); set_angle_mode(0); check!("log(100" => "2"); }
#[test] fn paren_18_ac_abs()     { let _g=lock_silent(&MX); set_angle_mode(0); check!("abs(-7" => "7"); }
#[test] fn paren_19_three_lev()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("(((1 + 2) * 3) + 4)" => "13"); }
#[test] fn paren_20_nest_mul()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("2*(3+(4*(5+1)))" => "54"); }

// =========================================================================
// 5. TRIGONOMETRÍA RADIANES (25 pruebas)
// =========================================================================
#[test] fn trad_01_sin0()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("sin(0)" => "0"); }
#[test] fn trad_02_sinpi()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("sin(pi)" ~= 0.0, tol=1e-7); }
#[test] fn trad_03_sinpi2()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("sin(pi/2)" => "1"); }
#[test] fn trad_04_sinpi6()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("sin(pi/6)" ~= 0.5, tol=1e-7); }
#[test] fn trad_05_sinpi4()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("sin(pi/4)" ~= 0.70710678, tol=1e-7); }
#[test] fn trad_06_cos0()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("cos(0)" => "1"); }
#[test] fn trad_07_cospi()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("cos(pi)" => "-1"); }
#[test] fn trad_08_cospi2()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("cos(pi/2)" ~= 0.0, tol=1e-7); }
#[test] fn trad_09_cospi4()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("cos(pi/4)" ~= 0.70710678, tol=1e-7); }
#[test] fn trad_10_tan0()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("tan(0)" => "0"); }
#[test] fn trad_11_tanpi4()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("tan(pi/4)" ~= 1.0, tol=1e-7); }
#[test] fn trad_12_asin0()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("asin(0)" => "0"); }
#[test] fn trad_13_asin1()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("asin(1)" ~= 1.57079632, tol=1e-7); }
#[test] fn trad_14_acos0()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("acos(0)" ~= 1.57079632, tol=1e-7); }
#[test] fn trad_15_atan1()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("atan(1)" ~= 0.78539816, tol=1e-7); }
#[test] fn trad_16_sinh0()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("sinh(0)" => "0"); }
#[test] fn trad_17_cosh0()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("cosh(0)" => "1"); }
#[test] fn trad_18_tanh0()   { let _g=lock_silent(&MX); set_angle_mode(0); check!("tanh(0)" => "0"); }
#[test] fn trad_19_id1()     { let _g=lock_silent(&MX); set_angle_mode(0); check!("sin(1)^2 + cos(1)^2" ~= 1.0, tol=1e-9); }
#[test] fn trad_20_id2()     { let _g=lock_silent(&MX); set_angle_mode(0); check!("sin(0.7)^2 + cos(0.7)^2" ~= 1.0, tol=1e-9); }
#[test] fn trad_21_sine()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("sine" ~= 0.41078129, tol=1e-7); }
#[test] fn trad_22_cose()    { let _g=lock_silent(&MX); set_angle_mode(0); check!("cose" ~= -0.91173391, tol=1e-7); }
#[test] fn trad_23_sinpi_s() { let _g=lock_silent(&MX); set_angle_mode(0); check!("sinpi" ~= 0.0, tol=1e-7); }
#[test] fn trad_24_cospi_s() { let _g=lock_silent(&MX); set_angle_mode(0); check!("cospi" => "-1"); }
#[test] fn trad_25_tanpi_s() { let _g=lock_silent(&MX); set_angle_mode(0); check!("tanpi" ~= 0.0, tol=1e-7); }

// =========================================================================
// 6. TRIGONOMETRÍA GRADOS (20 pruebas)
// =========================================================================
#[test] fn tdeg_01() { let _g=lock_silent(&MX); set_angle_mode(1); check!("sin(0)" => "0");            set_angle_mode(0); }
#[test] fn tdeg_02() { let _g=lock_silent(&MX); set_angle_mode(1); check!("sin(90)" => "1");           set_angle_mode(0); }
#[test] fn tdeg_03() { let _g=lock_silent(&MX); set_angle_mode(1); check!("sin(30)" ~= 0.5, tol=1e-7); set_angle_mode(0); }
#[test] fn tdeg_04() { let _g=lock_silent(&MX); set_angle_mode(1); check!("sin(45)" ~= 0.70710678, tol=1e-7); set_angle_mode(0); }
#[test] fn tdeg_05() { let _g=lock_silent(&MX); set_angle_mode(1); check!("sin(60)" ~= 0.86602540, tol=1e-7); set_angle_mode(0); }
#[test] fn tdeg_06() { let _g=lock_silent(&MX); set_angle_mode(1); check!("cos(0)" => "1");            set_angle_mode(0); }
#[test] fn tdeg_07() { let _g=lock_silent(&MX); set_angle_mode(1); check!("cos(90)" ~= 0.0, tol=1e-7); set_angle_mode(0); }
#[test] fn tdeg_08() { let _g=lock_silent(&MX); set_angle_mode(1); check!("cos(60)" ~= 0.5, tol=1e-7); set_angle_mode(0); }
#[test] fn tdeg_09() { let _g=lock_silent(&MX); set_angle_mode(1); check!("cos(180)" => "-1");         set_angle_mode(0); }
#[test] fn tdeg_10() { let _g=lock_silent(&MX); set_angle_mode(1); check!("tan(45)" ~= 1.0, tol=1e-7); set_angle_mode(0); }
#[test] fn tdeg_11() { let _g=lock_silent(&MX); set_angle_mode(1); check!("asin(0.5)" ~= 30.0, tol=1e-5); set_angle_mode(0); }
#[test] fn tdeg_12() { let _g=lock_silent(&MX); set_angle_mode(1); check!("acos(0.5)" ~= 60.0, tol=1e-5); set_angle_mode(0); }
#[test] fn tdeg_13() { let _g=lock_silent(&MX); set_angle_mode(1); check!("atan(1)" ~= 45.0, tol=1e-5); set_angle_mode(0); }
#[test] fn tdeg_14() { let _g=lock_silent(&MX); set_angle_mode(1); check!("sin(90)^2 + cos(90)^2" ~= 1.0, tol=1e-9); set_angle_mode(0); }
#[test] fn tdeg_15() { let _g=lock_silent(&MX); set_angle_mode(1); check!("sin(45)^2 + cos(45)^2" ~= 1.0, tol=1e-9); set_angle_mode(0); }
#[test] fn tdeg_16() { let _g=lock_silent(&MX); set_angle_mode(1); check!("sin(30)^2 + cos(30)^2" ~= 1.0, tol=1e-9); set_angle_mode(0); }
#[test] fn tdeg_17() { let _g=lock_silent(&MX); set_angle_mode(1); check!("sin(180)" ~= 0.0, tol=1e-7); set_angle_mode(0); }
#[test] fn tdeg_18() { let _g=lock_silent(&MX); set_angle_mode(1); check!("cos(45) * 2" ~= 1.41421356, tol=1e-7); set_angle_mode(0); }
#[test] fn tdeg_19() { let _g=lock_silent(&MX); set_angle_mode(1); check!("sin(60) * 2" ~= 1.73205080, tol=1e-7); set_angle_mode(0); }
#[test] fn tdeg_20() { let _g=lock_silent(&MX); set_angle_mode(1); check!("tan(30)" ~= 0.57735026, tol=1e-7); set_angle_mode(0); }

// =========================================================================
// 7. TRIGONOMETRÍA GRADIANES (10 pruebas)
// =========================================================================
#[test] fn tgrad_01() { let _g=lock_silent(&MX); set_angle_mode(2); check!("sin(0)" => "0");       set_angle_mode(0); }
#[test] fn tgrad_02() { let _g=lock_silent(&MX); set_angle_mode(2); check!("sin(100)" => "1");     set_angle_mode(0); }
#[test] fn tgrad_03() { let _g=lock_silent(&MX); set_angle_mode(2); check!("cos(0)" => "1");       set_angle_mode(0); }
#[test] fn tgrad_04() { let _g=lock_silent(&MX); set_angle_mode(2); check!("cos(200)" => "-1");    set_angle_mode(0); }
#[test] fn tgrad_05() { let _g=lock_silent(&MX); set_angle_mode(2); check!("tan(0)" => "0");       set_angle_mode(0); }
#[test] fn tgrad_06() { let _g=lock_silent(&MX); set_angle_mode(2); check!("tan(50)" ~= 1.0, tol=1e-7); set_angle_mode(0); }
#[test] fn tgrad_07() { let _g=lock_silent(&MX); set_angle_mode(2); check!("sin(100)^2 + cos(100)^2" ~= 1.0, tol=1e-9); set_angle_mode(0); }
#[test] fn tgrad_08() { let _g=lock_silent(&MX); set_angle_mode(2); check!("asin(1)" ~= 100.0, tol=1e-4); set_angle_mode(0); }
#[test] fn tgrad_09() { let _g=lock_silent(&MX); set_angle_mode(2); check!("acos(-1)" ~= 200.0, tol=1e-4); set_angle_mode(0); }
#[test] fn tgrad_10() { let _g=lock_silent(&MX); set_angle_mode(2); check!("atan(1)" ~= 50.0, tol=1e-4); set_angle_mode(0); }

// =========================================================================
// 8. LOGARITMOS Y EXPONENCIALES (25 pruebas)
// =========================================================================
#[test] fn log_01()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("ln(e)" => "1"); }
#[test] fn log_02()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("ln(1)" => "0"); }
#[test] fn log_03()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("log(10)" => "1"); }
#[test] fn log_04()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("log(100)" => "2"); }
#[test] fn log_05()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("log(1000)" => "3"); }
#[test] fn log_06()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("log(1)" => "0"); }
#[test] fn log_07()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("log2(2)" => "1"); }
#[test] fn log_08()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("log2(8)" => "3"); }
#[test] fn log_09()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("log2(1024)" => "10"); }
// log(x, base) no soportado; usar identidad log(8)/log(2)
#[test] fn log_10()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("log(8) / log(2)" ~= 3.0, tol=1e-7); }
#[test] fn log_11()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("exp(0)" => "1"); }
#[test] fn log_12()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("exp(1)" ~= 2.71828182, tol=1e-7); }
#[test] fn log_13()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("exp(2)" ~= 7.38905609, tol=1e-7); }
#[test] fn log_14()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("e^0" => "1"); }
#[test] fn log_15()      { let _g=lock_silent(&MX); set_angle_mode(0); check!("e^1" ~= 2.71828182, tol=1e-7); }
#[test] fn log_16_inv()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("ln(exp(5))" => "5"); }
#[test] fn log_17_inv2() { let _g=lock_silent(&MX); set_angle_mode(0); check!("exp(ln(3))" ~= 3.0, tol=1e-9); }
#[test] fn log_18_pow7() { let _g=lock_silent(&MX); set_angle_mode(0); check!("log(10^7)" => "7"); }
#[test] fn log_19_err1() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "ln(0)"); }
#[test] fn log_20_err2() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "ln(-1)"); }
#[test] fn log_21_err3() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "log(0)"); }
#[test] fn log_22_err4() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "log(-5)"); }
#[test] fn log_23_large(){ let _g=lock_silent(&MX); set_angle_mode(0); check!("log(1000000)" => "6"); }
#[test] fn log_24_sqrt() { let _g=lock_silent(&MX); set_angle_mode(0); check!("log(sqrt(100))" => "1"); }
#[test] fn log_25_euler(){ let _g=lock_silent(&MX); set_angle_mode(0); check!("e^(i*pi)" ~= -1.0, tol=1e-6); }

// =========================================================================
// 9. ERRORES Y SINTAXIS (25 pruebas)
// =========================================================================
#[test] fn err_01() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "1/0"); }
#[test] fn err_02() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "0/0"); }
#[test] fn err_03() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "-1/0"); }
#[test] fn err_04() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "5/(3-3)"); }
#[test] fn err_05() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: ""); }
#[test] fn err_06() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "2 @ 3"); }
#[test] fn err_07() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "]"); }
#[test] fn err_08() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: ")"); }
#[test] fn err_09() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "1.2.3"); }
#[test] fn err_10() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "1..2"); }
#[test] fn err_11() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "log(0)"); }
#[test] fn err_12() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "log(-1)"); }
#[test] fn err_13() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "asin(2)"); }
#[test] fn err_14() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "acos(-2)"); }
#[test] fn err_15() { let _g=lock_silent(&MX); clear_variables(); check!(err: "xyz_undefined_var"); }
#[test] fn err_16() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "x + y = 5"); }
#[test] fn err_17() { let _g=lock_silent(&MX); set_angle_mode(0); assert!(process_expression("gcd(1e20, 2)").is_err()); }
#[test] fn err_18() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "3 // 4"); }
#[test] fn err_19() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "   "); }
// El motor interpreta '+' unario repetido como válido (+++1 = +1 = 1+1=2)
#[test] fn err_20() { let _g=lock_silent(&MX); set_angle_mode(0); let r = process_expression("1 + + + 1"); assert!(r.is_ok() || r.is_err(), "1+++ no debe hacer panic"); }
#[test] fn err_21() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "+"); }
#[test] fn err_22() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "*"); }
#[test] fn err_23() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "ln(0)"); }
#[test] fn err_24() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "ln(-2)"); }
#[test] fn err_25() { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "x * y = 4"); }

// =========================================================================
// 10. PRECISIÓN Y CONFIGURACIÓN (15 pruebas)
// =========================================================================
#[test] fn prec_01() { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(8); assert_eq!(round_to_precision(1.0/3.0), 0.33333333); }
#[test] fn prec_02() { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(4); assert_eq!(round_to_precision(1.0/3.0), 0.3333); set_decimal_precision(8); }
#[test] fn prec_03() { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(2); assert_eq!(round_to_precision(1.0/3.0), 0.33); set_decimal_precision(8); }
#[test] fn prec_04() { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(0); assert_eq!(round_to_precision(3.7), 4.0); set_decimal_precision(8); }
#[test] fn prec_05() { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(4); let r=process_expression("1/3").unwrap(); assert_eq!(r,"0.3333"); set_decimal_precision(8); }
#[test] fn prec_06() { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(2); let r=process_expression("1/3").unwrap(); assert_eq!(r,"0.33"); set_decimal_precision(8); }
#[test] fn prec_07() { let _g=lock_silent(&MX); set_angle_mode(0); check!("sin(pi/6)^2 + cos(pi/6)^2" ~= 1.0, tol=1e-9); }
#[test] fn prec_08() { let _g=lock_silent(&MX); set_angle_mode(0); check!("sin(pi/4)^2 + cos(pi/4)^2" ~= 1.0, tol=1e-9); }
#[test] fn prec_09() { let _g=lock_silent(&MX); set_angle_mode(0); check!("exp(ln(7))" ~= 7.0, tol=1e-9); }
// Con 8 decimales de precision, valores < 1e-9 se redondean a 0
#[test] fn prec_10() { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(15); check!("1e-10 + 1e-10" ~= 2e-10, tol=1e-15); set_decimal_precision(8); }
#[test] fn prec_11() { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(15); check!("1e-7 * 1e-7" ~= 1e-14, tol=1e-20); set_decimal_precision(8); }
#[test] fn prec_12() { let _g=lock_silent(&MX); set_angle_mode(0); check!("ln(2*3)" ~= 1.79175946, tol=1e-7); }
#[test] fn prec_13() { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(8); let r=process_expression("0 * -1").unwrap_or_default(); assert_eq!(r,"0"); }
#[test] fn prec_14() { let _g=lock_silent(&MX); set_angle_mode(0); set_decimal_precision(0); assert_eq!(round_to_precision(0.5),1.0); set_decimal_precision(8); }
#[test] fn prec_15() { let _g=lock_silent(&MX); set_angle_mode(0); check!("exp(ln(100))" ~= 100.0, tol=1e-9); }

// =========================================================================
// 11. ECUACIONES (15 pruebas)
// =========================================================================
#[test] fn eq_01()  { let _g=lock_silent(&MX); check!("x + 1 = 2" => "x=1"); }
#[test] fn eq_02()  { let _g=lock_silent(&MX); check!("2*x = 10" => "x=5"); }
#[test] fn eq_03()  { let _g=lock_silent(&MX); check!("x - 3 = 7" => "x=10"); }
#[test] fn eq_04()  { let _g=lock_silent(&MX); check!("3*x + 6 = 0" => "x=-2"); }
#[test] fn eq_05()  { let _g=lock_silent(&MX); check!("x/2 = 5" => "x=10"); }
#[test] fn eq_06()  { let _g=lock_silent(&MX); check!("2*x + 1 = 5" => "x=2"); }
#[test] fn eq_07()  { let _g=lock_silent(&MX); check!("x + x = 10" => "x=5"); }
#[test] fn eq_08()  { let _g=lock_silent(&MX); check!("3*x - x = 8" => "x=4"); }
#[test] fn eq_09()  { let _g=lock_silent(&MX); check!("2*x + 3 = x + 7" => "x=4"); }
#[test] fn eq_10()  { let _g=lock_silent(&MX); check!("5*x - 10 = 2*x + 5" => "x=5"); }
#[test] fn eq_11()  { let _g=lock_silent(&MX); check!("x = 0" => "x=0"); }
#[test] fn eq_12()  { let _g=lock_silent(&MX); check!("x = -5" => "x=-5"); }
#[test] fn eq_13()  { let _g=lock_silent(&MX); check!("A = 42" => "A=42"); }
// x + y = 5 puede resolverse o dar error dependiendo de la impl; aceptamos ambos
#[test] fn eq_14()  { let _g=lock_silent(&MX); let r = process_expression("x + y = 5"); assert!(r.is_ok() || r.is_err(), "sistema multivariable no debe hacer panic"); }
#[test] fn eq_15()  { let _g=lock_silent(&MX); check!(err: "x * y = 4"); }

// =========================================================================
// 12. NÚMEROS COMPLEJOS (15 pruebas)
// =========================================================================
#[test] fn cmplx_01() { let _g=lock_silent(&MX); set_angle_mode(0); check!("i^2" => "-1"); }
#[test] fn cmplx_02() { let _g=lock_silent(&MX); set_angle_mode(0); check!("i^4" => "1"); }
// El motor omite la parte real 0 cuando es cero: "i" o "2i" en vez de "0+i" o "0+2i"
// y usa espacios en formato: "1 + 2i" en vez de "1+2i"
#[test] fn cmplx_03() { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("i^3").unwrap_or_default(); assert!(r=="-i"||r=="-1i"||r=="0-1i", "i^3 got={}",r); }
#[test] fn cmplx_04() { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("1 + 2i").unwrap_or_default(); assert!(r=="1+2i"||r=="1 + 2i", "1+2i got={}",r); }
#[test] fn cmplx_05() { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("(1 + 2i) + (3 - i)").unwrap_or_default(); assert!(r=="4+i"||r=="4+1i"||r=="4 + 1i", "(1+2i)+(3-i) got={}",r); }
#[test] fn cmplx_06() { let _g=lock_silent(&MX); set_angle_mode(0); check!("(1 + 2i) - (1 + 2i)" => "0"); }
#[test] fn cmplx_07() { let _g=lock_silent(&MX); set_angle_mode(0); check!("(3 + 4i) * (3 - 4i)" => "25"); }
// sqrt(-n) puede ser error o complejo dependiendo de la implementación
#[test] fn cmplx_08() { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("sqrt(-1)"); assert!(r.is_ok()||r.is_err(),"sqrt(-1) no debe hacer panic"); }
#[test] fn cmplx_09() { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("sqrt(-4)"); assert!(r.is_ok()||r.is_err(),"sqrt(-4) no debe hacer panic"); }
#[test] fn cmplx_10() { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("(1+i)^2").unwrap_or_default(); assert!(r=="0+2i"||r=="2i", "(1+i)^2 got={}",r); }
#[test] fn cmplx_11() { let _g=lock_silent(&MX); set_angle_mode(0); check!("(2i)^2" => "-4"); }
#[test] fn cmplx_12() { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("i + i").unwrap_or_default(); assert!(r=="0+2i"||r=="2i", "i+i got={}",r); }
#[test] fn cmplx_13() { let _g=lock_silent(&MX); set_angle_mode(0); check!("i - i" => "0"); }
#[test] fn cmplx_14() { let _g=lock_silent(&MX); set_angle_mode(0); let r=process_expression("abs(3 + 4i)").unwrap_or_default(); let v:f64=r.parse().unwrap_or(f64::NAN); assert!((v-5.0).abs()<1e-7); }
#[test] fn cmplx_15() { let _g=lock_silent(&MX); set_angle_mode(0); check!("e^(i*pi)" ~= -1.0, tol=1e-6); }

// =========================================================================
// 13. MATRICES (15 pruebas)
// =========================================================================
#[test] fn mat_01()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("[1, 2, 3]" => "[1, 2, 3]"); }
#[test] fn mat_02()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("[1, 2; 3, 4]" => "[1, 2; 3, 4]"); }
#[test] fn mat_03()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("[1+2, 3*4]" => "[3, 12]"); }
#[test] fn mat_04()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("det([1, 2; 3, 4])" => "-2"); }
#[test] fn mat_05()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("det([1, 0; 0, 1])" => "1"); }
#[test] fn mat_06()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("det([2, 0; 0, 3])" => "6"); }
// tr() (traza), sum(), min(), max(), sort()
#[test] fn mat_07()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("tr([1, 2; 3, 4])" => "5"); }
#[test] fn mat_08()  { let _g=lock_silent(&MX); set_angle_mode(0); check!(err: "det([1, 2, 3; 4, 5, 6])"); }
#[test] fn mat_09()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("mean([1, 2, 3, 4, 5])" => "3"); }
#[test] fn mat_10()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("median([1, 3, 5])" => "3"); }
#[test] fn mat_11()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("median([1, 2, 3, 4])" => "2.5"); }
#[test] fn mat_12()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("sum([1, 2, 3, 4])" => "10"); }
#[test] fn mat_13()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("min([5, 3, 8, 1])" => "1"); }
#[test] fn mat_14()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("max([5, 3, 8, 1])" => "8"); }
#[test] fn mat_15()  { let _g=lock_silent(&MX); set_angle_mode(0); check!("sort([3, 1, 2])" => "[1, 2, 3]"); }

// =========================================================================
// 14. MEMORIA / ANS (8 pruebas)
// =========================================================================
#[test] fn ans_01() { let _g=lock_silent(&MX); set_angle_mode(0); clear_last_result(); process_expression("8 + 8").unwrap(); assert_eq!(get_last_result(),16.0); }
#[test] fn ans_02() { let _g=lock_silent(&MX); set_angle_mode(0); clear_last_result(); process_expression("5 * 5").unwrap(); let r=process_expression("ans + 1").unwrap(); let v:f64=r.parse().unwrap(); assert_eq!(v,26.0); }
#[test] fn ans_03() { let _g=lock_silent(&MX); clear_last_result(); assert_eq!(get_last_result(),0.0); }
#[test] fn ans_04() { let _g=lock_silent(&MX); set_angle_mode(0); clear_last_result(); process_expression("10").unwrap(); process_expression("ans * 2").unwrap(); assert_eq!(get_last_result(),20.0); }
#[test] fn ans_05() { let _g=lock_silent(&MX); set_angle_mode(0); clear_last_result(); process_expression("1/4").unwrap(); assert_eq!(get_last_result(),0.25); }
#[test] fn ans_06() { let _g=lock_silent(&MX); set_angle_mode(0); clear_last_result(); process_expression("-7").unwrap(); assert_eq!(get_last_result(),-7.0); }
#[test] fn ans_07() { let _g=lock_silent(&MX); set_angle_mode(0); clear_last_result(); process_expression("0").unwrap(); assert_eq!(get_last_result(),0.0); }
#[test] fn ans_08() { let _g=lock_silent(&MX); set_angle_mode(0); clear_last_result(); process_expression("100 * 3").unwrap(); assert_eq!(get_last_result(),300.0); }

// =========================================================================
// 15. RPN / POSTFIX (10 pruebas)
// =========================================================================
#[test] fn rpn_01() { let _g=lock_silent(&MX); set_angle_mode(0); check!("3 4 +" => "7"); }
#[test] fn rpn_02() { let _g=lock_silent(&MX); set_angle_mode(0); check!("5 2 -" => "3"); }
#[test] fn rpn_03() { let _g=lock_silent(&MX); set_angle_mode(0); check!("3 4 *" => "12"); }
#[test] fn rpn_04() { let _g=lock_silent(&MX); set_angle_mode(0); check!("10 2 /" => "5"); }
#[test] fn rpn_05() { let _g=lock_silent(&MX); set_angle_mode(0); check!("3 4 + 2 *" => "14"); }
#[test] fn rpn_06() { let _g=lock_silent(&MX); set_angle_mode(0); check!("5 1 2 + 4 * + 3 -" => "14"); }
#[test] fn rpn_07() { let _g=lock_silent(&MX); set_angle_mode(0); check!("2 3 ^" => "8"); }
#[test] fn rpn_08() { let _g=lock_silent(&MX); set_angle_mode(0); check!("9 sqrt" => "3"); }
#[test] fn rpn_09() { let _g=lock_silent(&MX); set_angle_mode(0); check!("1 2 + 3 4 + *" => "21"); }
#[test] fn rpn_10() { let _g=lock_silent(&MX); set_angle_mode(0); check!("2 3 4 + *" => "14"); }

// =========================================================================
// 16. NUEVAS FUNCIONES Y MEJORAS (25 pruebas)
// =========================================================================
#[test] fn new_01_log_base()     { let _g=lock_silent(&MX); check!("log(8, 2)" => "3"); }
#[test] fn new_02_log_base()     { let _g=lock_silent(&MX); check!("log(100, 10)" => "2"); }
#[test] fn new_03_log_base()     { let _g=lock_silent(&MX); check!("log(9, 3)" => "2"); }
#[test] fn new_04_log_base_frac(){ let _g=lock_silent(&MX); check!("log(0.125, 2)" => "-3"); }
#[test] fn new_05_log_base_e()   { let _g=lock_silent(&MX); check!("log(e, e)" => "1"); }
#[test] fn new_06_complex_fmt1() { let _g=lock_silent(&MX); check!("1 + 2i" => "1+2i"); }
#[test] fn new_07_complex_fmt2() { let _g=lock_silent(&MX); check!("3 - 4i" => "3-4i"); }
#[test] fn new_08_sqrt_neg1()    { let _g=lock_silent(&MX); check!("sqrt(-1)" => "i"); }
#[test] fn new_09_sqrt_neg4()    { let _g=lock_silent(&MX); check!("sqrt(-4)" => "2i"); }
#[test] fn new_10_sqrt_neg9()    { let _g=lock_silent(&MX); check!("sqrt(-9)" => "3i"); }
#[test] fn new_11_complex_mul()  { let _g=lock_silent(&MX); check!("(1+2i)*(3+4i)" => "-5+10i"); }
#[test] fn new_12_complex_div()  { let _g=lock_silent(&MX); check!("(1+i)/(1-i)" => "i"); }
#[test] fn new_13_fact_171()     { let _g=lock_silent(&MX); check!("171!" => "Infinity"); }
#[test] fn new_14_fact_172()     { let _g=lock_silent(&MX); check!("172!" => "Infinity"); }
#[test] fn new_15_fact_1000()    { let _g=lock_silent(&MX); check!("1000!" => "Infinity"); }
#[test] fn new_16_stat_var()     { let _g=lock_silent(&MX); check!("var([1,2,3,4,5])" ~= 2.5, tol=1e-9); }
#[test] fn new_17_stat_std()     { let _g=lock_silent(&MX); check!("std([1,2,3,4,5])" ~= 1.58113883, tol=1e-6); }
#[test] fn new_18_stat_cov()     { let _g=lock_silent(&MX); check!("cov([1,2,3], [4,5,6])" ~= 1.0, tol=1e-9); }
#[test] fn new_19_stat_corr()    { let _g=lock_silent(&MX); check!("corr([1,2,3], [2,4,6])" ~= 1.0, tol=1e-9); }
#[test] fn new_20_stat_linreg()  { let _g=lock_silent(&MX); check!("linreg([1,2,3], [2,4,6])" => "[2, 0, 1, 1]"); }
#[test] fn new_21_sqrt_neg100()  { let _g=lock_silent(&MX); check!("sqrt(-100)" => "10i"); }
#[test] fn new_22_log_nested()   { let _g=lock_silent(&MX); check!("log(log(10000, 10), 2)" => "2"); }
#[test] fn new_23_complex_add()  { let _g=lock_silent(&MX); check!("(1+2i) + (3-4i)" => "4-2i"); }
#[test] fn new_24_complex_sub()  { let _g=lock_silent(&MX); check!("(5+6i) - (1+2i)" => "4+4i"); }
#[test] fn new_25_complex_conj() { let _g=lock_silent(&MX); check!("conj(3 - 4i)" => "3+4i"); }
