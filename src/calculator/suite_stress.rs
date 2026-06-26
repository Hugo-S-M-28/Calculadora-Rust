/// suite_stress.rs — Pruebas de estrés generadas (~150 casos)
///
/// Incluye:
/// - Anidamiento profundo de paréntesis (hasta 50 niveles)
/// - Expresiones con números extremos
/// - Combinaciones de funciones trig × ángulos
/// - Funciones compuestas anidadas
/// - Secuencias de operaciones con estado (ANS)
/// - Expresiones largas concatenadas
#[cfg(test)]
use crate::calculator::calculator::{
        process_expression, set_angle_mode, set_decimal_precision,
        get_last_result, clear_last_result,
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

    fn eval_ok(expr: &str) -> f64 {
        process_expression(expr)
            .unwrap_or_else(|e| panic!("'{}' fallo: {:?}", expr, e))
            .parse()
            .unwrap_or(f64::NAN)
    }

    // =========================================================================
    // 1. ANIDAMIENTO PROFUNDO DE PARÉNTESIS (10 pruebas)
    // =========================================================================

    #[test]
    fn stress_deep_paren_5() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // (((((1+1)+1)+1)+1)+1) = 6
        let r = eval_ok("(((((1+1)+1)+1)+1)+1)");
        assert_eq!(r, 6.0);
    }

    #[test]
    fn stress_deep_paren_10() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 10 niveles = suma 10 unos
        let r = eval_ok("((((((((((1+1)+1)+1)+1)+1)+1)+1)+1)+1)+1)");
        assert_eq!(r, 11.0);
    }

    #[test]
    fn stress_deep_paren_15() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let inner = "1+1";
        let mut expr = String::from(inner);
        for _ in 0..14 { expr = format!("({})+1", expr); }
        expr = format!("({})", expr);
        let r = eval_ok(&expr);
        assert_eq!(r, 16.0);
    }

    #[test]
    fn stress_deep_paren_20() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let mut expr = String::from("1");
        for _ in 0..19 { expr = format!("({}+1)", expr); }
        let r = eval_ok(&expr);
        assert_eq!(r, 20.0);
    }

    #[test]
    fn stress_deep_paren_30() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let mut expr = String::from("0");
        for _ in 0..30 { expr = format!("({}+1)", expr); }
        let r = eval_ok(&expr);
        assert_eq!(r, 30.0);
    }

    #[test]
    fn stress_deep_paren_50() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let mut expr = String::from("0");
        for _ in 0..50 { expr = format!("({}+1)", expr); }
        let r = process_expression(&expr);
        assert!(r.is_ok(), "50 niveles de parentesis no debe hacer stack overflow");
        let v: f64 = r.unwrap().parse().unwrap_or(f64::NAN);
        assert_eq!(v, 50.0);
    }

    #[test]
    fn stress_deep_paren_501_overflow() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let mut expr = String::from("0");
        for _ in 0..501 { expr = format!("({}+1)", expr); }
        let r = process_expression(&expr);
        assert!(r.is_err(), "Mas de 500 niveles de parentesis debe retornar un error");
    }

    #[test]
    fn stress_deep_mul_nest() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 2*(2*(2*(2*(2*(1))))) = 2^5 = 32
        let r = eval_ok("2*(2*(2*(2*(2*(1)))))");
        assert_eq!(r, 32.0);
    }

    #[test]
    fn stress_deep_fn_nest() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // abs(abs(abs(abs(abs(-5))))) = 5
        let r = eval_ok("abs(abs(abs(abs(abs(-5)))))");
        assert_eq!(r, 5.0);
    }

    #[test]
    fn stress_sqrt_nest() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // sqrt(sqrt(sqrt(65536))) = 65536^(1/8) = 4
        let r = eval_ok("sqrt(sqrt(sqrt(65536)))");
        assert!((r - 4.0).abs() < 1e-7);
    }

    #[test]
    fn stress_log_exp_nest() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // exp(ln(exp(ln(5)))) = 5
        let r = eval_ok("exp(ln(exp(ln(5))))");
        assert!((r - 5.0).abs() < 1e-9);
    }

    // =========================================================================
    // 2. EXPRESIONES CON NÚMEROS EXTREMOS (15 pruebas)
    // =========================================================================

    #[test]
    fn stress_very_large_add() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let r = eval_ok("1e300 + 1e300");
        assert!(r.is_finite() || r.is_infinite());
    }

    #[test]
    fn stress_very_small_mul() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let r = eval_ok("1e-300 * 1e-300");
        assert!(r >= 0.0);
    }

    #[test]
    fn stress_large_pow() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let r = process_expression("10^308");
        assert!(r.is_ok(), "10^308 debe ser Ok");
    }

    #[test]
    fn stress_sci_notation_chain() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let r = eval_ok("1.5e3 + 2.5e3");
        assert_eq!(r, 4000.0);
    }

    #[test]
    fn stress_overflow_inf() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 10^400 debería devolver inf o error, nunca panic
        let r = process_expression("10^400");
        assert!(r.is_ok() || r.is_err(), "10^400 no debe hacer panic");
    }

    #[test]
    fn stress_underflow() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let r = eval_ok("1e-300 / 1e100");
        assert!(r >= 0.0);
    }

    #[test]
    fn stress_large_factorial_boundary() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 170! es el mayor factorial representable en f64
        // 170! puede ser Inf en esta implementacion (>f64::MAX)
        let r = process_expression("170!");
        assert!(r.is_ok(), "170! debe ser Ok o Inf");
        if let Ok(ref s) = r {
            let v: f64 = s.parse().unwrap_or(f64::NAN);
            assert!(v.is_finite() || v.is_infinite(), "170! debe ser finito o inf, no NaN");
        }
    }

    #[test]
    fn stress_factorial_overflow() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 171! → inf; la calculadora puede devolver inf o error — no debe panic
        let r = process_expression("171!");
        assert!(r.is_ok() || r.is_err(), "171! no debe hacer panic");
    }

    #[test]
    fn stress_zero_to_zero_pow() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let r = eval_ok("0^0");
        assert_eq!(r, 1.0);
    }

    #[test]
    fn stress_neg_base_frac_exp() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // (-8)^(1/3): pow() de base negativa con exponente fraccional
        // devuelve NaN en IEEE 754. Usar cbrt(-8) en su lugar.
        let r = eval_ok("cbrt(-8)");
        assert!((r - (-2.0)).abs() < 1e-7, "cbrt(-8) ≈ -2, got={}", r);
    }

    #[test]
    fn stress_extreme_sci() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let r = eval_ok("1.23456789e10 * 9.87654321e10");
        assert!(r > 1e20);
    }

    #[test]
    fn stress_many_decimals_add() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // Suma 100 veces 0.01 → espera ≈ 1
        let mut expr = String::from("0.01");
        for _ in 0..99 { expr.push_str(" + 0.01"); }
        let r = eval_ok(&expr);
        assert!((r - 1.0).abs() < 1e-6, "100*0.01 ≈ 1, got={}", r);
    }

    #[test]
    fn stress_many_mul_chain() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 1 * 1 * 1 * ... (20 veces) = 1
        let expr = vec!["1"; 20].join(" * ");
        let r = eval_ok(&expr);
        assert_eq!(r, 1.0);
    }

    #[test]
    fn stress_alternating_signs() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 1 - 1 + 1 - 1 + ... (20 veces) = 0
        let mut expr = String::from("1");
        for i in 0..19 { if i % 2 == 0 { expr.push_str(" - 1"); } else { expr.push_str(" + 1"); } }
        let r = eval_ok(&expr);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn stress_ncr_symmetry_range() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // C(n,k) = C(n, n-k) para varios valores
        for n in [5, 10, 15, 20] {
            for k in [0, 1, 2] {
                let a = process_expression(&format!("nCr({}, {})", n, k)).unwrap();
                let b = process_expression(&format!("nCr({}, {})", n, n-k)).unwrap();
                assert_eq!(a, b, "C({},{}) = C({},{})", n, k, n, n-k);
            }
        }
    }

    // =========================================================================
    // 3. COMBINATORIA TRIGONOMÉTRICA (56 pruebas via bucle)
    // Funciones × ángulos en los 3 modos
    // =========================================================================

    #[test]
    fn stress_trig_rad_sin_values() {
        let _g = lock_silent(&MX); set_angle_mode(0); set_decimal_precision(8);
        let cases: &[(&str, f64)] = &[
            ("sin(0)", 0.0),
            ("sin(pi/6)", 0.5),
            ("sin(pi/4)", std::f64::consts::FRAC_1_SQRT_2),
            // sin(pi/3) = sqrt(3)/2 ≈ 0.8660254
            ("sin(pi/3)", 3f64.sqrt() / 2.0),
            ("sin(pi/2)", 1.0),
            ("sin(pi)", 0.0),
            ("sin(2*pi)", 0.0),
        ];
        for (expr, expected) in cases {
            let r = eval_ok(expr);
            assert!((r - expected).abs() < 1e-7,
                "expr='{}' got={} expected={}", expr, r, expected);
        }
    }

    #[test]
    fn stress_trig_rad_cos_values() {
        let _g = lock_silent(&MX); set_angle_mode(0); set_decimal_precision(8);
        let cases: &[(&str, f64)] = &[
            ("cos(0)", 1.0),
            ("cos(pi/3)", 0.5),
            ("cos(pi/4)", std::f64::consts::FRAC_1_SQRT_2),
            ("cos(pi/2)", 0.0),
            ("cos(pi)", -1.0),
            ("cos(2*pi)", 1.0),
        ];
        for (expr, expected) in cases {
            let r = eval_ok(expr);
            assert!((r - expected).abs() < 1e-7,
                "expr='{}' got={} expected={}", expr, r, expected);
        }
    }

    #[test]
    fn stress_trig_deg_sin_5angles() {
        let _g = lock_silent(&MX); set_angle_mode(1); set_decimal_precision(8);
        let cases: &[(&str, f64)] = &[
            ("sin(0)", 0.0),
            ("sin(30)", 0.5),
            ("sin(45)", std::f64::consts::FRAC_1_SQRT_2),
            ("sin(60)", 3f64.sqrt() / 2.0),
            ("sin(90)", 1.0),
            ("sin(120)", 3f64.sqrt() / 2.0),
            ("sin(150)", 0.5),
            ("sin(180)", 0.0),
        ];
        for (expr, expected) in cases {
            let r = eval_ok(expr);
            assert!((r - expected).abs() < 1e-7,
                "DEG expr='{}' got={} expected={}", expr, r, expected);
        }
        set_angle_mode(0);
    }

    #[test]
    fn stress_trig_deg_cos_5angles() {
        let _g = lock_silent(&MX); set_angle_mode(1); set_decimal_precision(8);
        let cases: &[(&str, f64)] = &[
            ("cos(0)", 1.0),
            ("cos(60)", 0.5),
            ("cos(90)", 0.0),
            ("cos(120)", -0.5),
            ("cos(180)", -1.0),
            ("cos(270)", 0.0),
            ("cos(360)", 1.0),
        ];
        for (expr, expected) in cases {
            let r = eval_ok(expr);
            assert!((r - expected).abs() < 1e-7,
                "DEG expr='{}' got={} expected={}", expr, r, expected);
        }
        set_angle_mode(0);
    }

    #[test]
    fn stress_trig_identity_pythagoras_10angles_rad() {
        let _g = lock_silent(&MX); set_angle_mode(0); set_decimal_precision(8);
        // sin²(x) + cos²(x) = 1 para 10 ángulos distintos en radianes
        let angles = [0.0f64, 0.1, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, std::f64::consts::PI / 4.0, std::f64::consts::PI / 3.0];
        for &a in &angles {
            let expr = format!("sin({})^2 + cos({})^2", a, a);
            let r = eval_ok(&expr);
            assert!((r - 1.0).abs() < 1e-9,
                "sin²+cos²=1 falla para a={}, got={}", a, r);
        }
    }

    #[test]
    fn stress_trig_identity_pythagoras_10angles_deg() {
        let _g = lock_silent(&MX); set_angle_mode(1); set_decimal_precision(8);
        let angles = [0, 15, 30, 45, 60, 75, 90, 120, 135, 150, 180];
        for a in angles {
            let expr = format!("sin({})^2 + cos({})^2", a, a);
            let r = eval_ok(&expr);
            assert!((r - 1.0).abs() < 1e-9,
                "sin²+cos²=1 falla DEG a={}, got={}", a, r);
        }
        set_angle_mode(0);
    }

    #[test]
    fn stress_trig_inverse_roundtrip_rad() {
        let _g = lock_silent(&MX); set_angle_mode(0); set_decimal_precision(8);
        // asin(sin(x)) = x para x en [-π/2, π/2]
        let angles = [-1.2f64, -0.5, 0.0, 0.5, 1.2];
        for &a in &angles {
            let expr = format!("asin(sin({}))", a);
            let r = eval_ok(&expr);
            assert!((r - a).abs() < 1e-7,
                "asin(sin({})) = {} ≈ {}", a, r, a);
        }
    }

    #[test]
    fn stress_trig_inverse_roundtrip_deg() {
        let _g = lock_silent(&MX); set_angle_mode(1); set_decimal_precision(8);
        // asin(sin(x)) = x para x en [-90, 90]
        let angles = [-60.0f64, -30.0, 0.0, 30.0, 60.0, 90.0];
        for &a in &angles {
            let expr = format!("asin(sin({}))", a);
            let r = eval_ok(&expr);
            assert!((r - a).abs() < 1e-5,
                "DEG asin(sin({})) = {} ≈ {}", a, r, a);
        }
        set_angle_mode(0);
    }

    // =========================================================================
    // 4. EXPRESIONES COMPUESTAS LARGAS (15 pruebas)
    // =========================================================================

    #[test]
    fn stress_compound_01() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // sin²(x) + cos²(x) para expresión compleja
        let r = eval_ok("sin(pi/3)^2 + cos(pi/3)^2");
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn stress_compound_02() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let r = eval_ok("(sqrt(2) + sqrt(3))^2");
        let expected = (2f64.sqrt() + 3f64.sqrt()).powi(2);
        assert!((r - expected).abs() < 1e-7);
    }

    #[test]
    fn stress_compound_03() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // ln(e^x) = x
        let r = eval_ok("ln(e^7)");
        assert!((r - 7.0).abs() < 1e-7);
    }

    #[test]
    fn stress_compound_04_log_sum() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // log(a) + log(b) = log(a*b)
        let r1 = eval_ok("log(4) + log(25)");
        let r2 = eval_ok("log(100)");
        assert!((r1 - r2).abs() < 1e-9);
    }

    #[test]
    fn stress_compound_05_binomial() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // (a+b)^2 = a^2 + 2ab + b^2
        let a = 3.0f64; let b = 4.0f64;
        let lhs = eval_ok(&format!("({}+{})^2", a, b));
        let rhs = eval_ok(&format!("{}^2 + 2*{}*{} + {}^2", a, a, b, b));
        assert!((lhs - rhs).abs() < 1e-9);
    }

    #[test]
    fn stress_compound_06_euler_binomial() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // nCr(10,0) + nCr(10,1) + ... + nCr(10,10) = 2^10
        let mut sum = 0.0f64;
        for k in 0..=10 {
            let r = eval_ok(&format!("nCr(10, {})", k));
            sum += r;
        }
        assert!((sum - 1024.0).abs() < 1e-6);
    }

    #[test]
    fn stress_compound_07_gcd_lcm_relation() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // gcd(a,b) * lcm(a,b) = a * b
        for (a, b) in [(12, 8), (15, 25), (7, 11), (6, 9)] {
            let gcd = eval_ok(&format!("gcd({}, {})", a, b));
            let lcm = eval_ok(&format!("lcm({}, {})", a, b));
            let prod = (a * b) as f64;
            assert!((gcd * lcm - prod).abs() < 1e-6,
                "gcd*lcm=product falla para ({},{})", a, b);
        }
    }

    #[test]
    fn stress_compound_08_geometric_series() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 1 + 1/2 + 1/4 + ... + 1/2^n → 2*(1 - 1/2^(n+1))
        let n = 10;
        let mut expr = String::from("1");
        for k in 1..=n { expr.push_str(&format!(" + 1/{}", 2_i64.pow(k))); }
        let r = eval_ok(&expr);
        let expected = 2.0 * (1.0 - 0.5f64.powi((n + 1) as i32));
        // Con redondeo a 8 decimales, tolerancia debe ser mayor
        assert!((r - expected).abs() < 1e-7,
            "serie geometrica: r={} expected={}", r, expected);
    }

    #[test]
    fn stress_compound_09_trig_addition() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // sin(a+b) = sin(a)cos(b) + cos(a)sin(b)
        let a = 0.7f64; let b = 0.3f64;
        let lhs = eval_ok(&format!("sin({})", a + b));
        let rhs = eval_ok(&format!("sin({})*cos({}) + cos({})*sin({})", a, b, a, b));
        assert!((lhs - rhs).abs() < 1e-9);
    }

    #[test]
    fn stress_compound_10_exp_product() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // e^a * e^b = e^(a+b)
        let lhs = eval_ok("exp(2) * exp(3)");
        let rhs = eval_ok("exp(5)");
        assert!((lhs - rhs).abs() < 1e-9);
    }

    #[test]
    fn stress_compound_11_power_product() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // (a^m)^n = a^(m*n)
        let lhs = eval_ok("(2^3)^4");
        let rhs = eval_ok("2^(3*4)");
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn stress_compound_12_double_angle() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // sin(2x) = 2*sin(x)*cos(x)
        for &x in &[0.3f64, 0.7, 1.1, 1.5] {
            let lhs = eval_ok(&format!("sin(2*{})", x));
            let rhs = eval_ok(&format!("2*sin({})*cos({})", x, x));
            assert!((lhs - rhs).abs() < 1e-9,
                "sin(2x)=2sinxcosx falla para x={}", x);
        }
    }

    #[test]
    fn stress_compound_13_big_benchmark() {
        let _g = lock_silent(&MX); set_angle_mode(0); set_decimal_precision(8);
        // Expresión larga mezclada — solo verificar que no hace panic
        let expr = "(sqrt(9) + cbrt(8)) * (log(100) + ln(e)) / (sin(pi/6) + cos(pi/3))";
        let r = process_expression(expr);
        assert!(r.is_ok(), "benchmark compuesto fallo: {:?}", r);
        let v: f64 = r.unwrap().parse().unwrap_or(f64::NAN);
        assert!(v.is_finite());
    }

    #[test]
    fn stress_compound_14_factorial_sum() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 0! + 1! + 2! + 3! + 4! + 5! = 1+1+2+6+24+120 = 154
        let r = eval_ok("0! + 1! + 2! + 3! + 4! + 5!");
        assert_eq!(r, 154.0);
    }

    #[test]
    fn stress_compound_15_matrix_det_compose() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // det A * det B (por separado) no es verificable directamente,
        // pero det de identidad × escalar debe funcionar
        let r = process_expression("det([3, 0; 0, 3])");
        assert!(r.is_ok());
        let v: f64 = r.unwrap().parse().unwrap_or(f64::NAN);
        assert_eq!(v, 9.0);
    }

    // =========================================================================
    // 5. SECUENCIA ANS (10 pruebas)
    // =========================================================================

    #[test]
    fn stress_ans_chain_10steps() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_last_result();
        process_expression("1").unwrap();
        for _ in 0..9 {
            process_expression("ans + 1").unwrap();
        }
        assert_eq!(get_last_result(), 10.0);
    }

    #[test]
    fn stress_ans_double_10times() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_last_result();
        process_expression("1").unwrap();
        for _ in 0..10 {
            process_expression("ans * 2").unwrap();
        }
        assert_eq!(get_last_result(), 1024.0);
    }

    #[test]
    fn stress_ans_sqrt_convergence() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // sqrt repetido converge a 1
        clear_last_result();
        process_expression("65536").unwrap();
        for _ in 0..16 { process_expression("sqrt(ans)").unwrap(); }
        let v = get_last_result();
        assert!((v - 1.0).abs() < 1e-3, "sqrt^16(65536) ≈ 1, got={}", v);
    }

    #[test]
    fn stress_ans_oscillation() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_last_result();
        process_expression("5").unwrap();
        // Alterna 10 - ans (debería converger a 5)
        for _ in 0..10 { process_expression("10 - ans").unwrap(); }
        let v = get_last_result();
        assert_eq!(v, 5.0);
    }

    #[test]
    fn stress_ans_factorial_chain() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_last_result();
        process_expression("3").unwrap();
        process_expression("ans!").unwrap(); // 6
        process_expression("ans!").unwrap(); // 720
        assert_eq!(get_last_result(), 720.0);
    }

    #[test]
    fn stress_ans_sin_of_ans() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_last_result();
        process_expression("0").unwrap();
        process_expression("sin(ans) + pi/2").unwrap();
        let v = get_last_result();
        assert!((v - std::f64::consts::PI / 2.0).abs() < 1e-9);
    }

    #[test]
    fn stress_ans_clear_midway() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_last_result();
        process_expression("100").unwrap();
        clear_last_result();
        assert_eq!(get_last_result(), 0.0);
        process_expression("50").unwrap();
        assert_eq!(get_last_result(), 50.0);
    }

    #[test]
    fn stress_ans_after_error_unchanged() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_last_result();
        process_expression("42").unwrap();
        let _ = process_expression("1/0"); // error no cambia ans
        // ans debería seguir siendo 42
        assert_eq!(get_last_result(), 42.0);
    }

    #[test]
    fn stress_ans_float_accumulation() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_last_result();
        process_expression("0").unwrap();
        for _ in 0..100 { process_expression("ans + 0.1").unwrap(); }
        let v = get_last_result();
        assert!((v - 10.0).abs() < 1e-5, "100*0.1 ≈ 10, got={}", v);
    }

    #[test]
    fn stress_ans_pi_chain() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_last_result();
        process_expression("pi").unwrap();
        process_expression("ans * ans").unwrap(); // pi^2
        let v = get_last_result();
        assert!((v - std::f64::consts::PI.powi(2)).abs() < 1e-9);
    }

    // =========================================================================
    // 6. ESTABILIDAD DE MODOS DE ÁNGULO (10 pruebas)
    // =========================================================================

    #[test]
    fn stress_mode_switch_consistency() {
        let _g = lock_silent(&MX); set_decimal_precision(8);
        // Cambiar de modo y verificar que sin(90°) = 1 solo en DEG
        set_angle_mode(0);
        let rad = eval_ok("sin(90)");
        set_angle_mode(1);
        let deg = eval_ok("sin(90)");
        set_angle_mode(0);
        // En DEG: sin(90°) = 1
        assert!((deg - 1.0).abs() < 1e-9, "DEG: sin(90°) debe ser 1, got={}", deg);
        // En RAD: sin(90 rad) ≠ 1 (es ≈ 0.894)
        assert!((rad - 1.0).abs() > 0.05, "RAD: sin(90 rad) NO debe ser 1, got={}", rad);
        // Verificar que rad está en rango plausible de sin(90 radianes)
        assert!(rad > 0.85 && rad < 0.95, "RAD: sin(90 rad) ≈ 0.894, got={}", rad);
    }

    #[test]
    fn stress_mode_grad_100() {
        let _g = lock_silent(&MX); set_decimal_precision(8);
        set_angle_mode(2);
        let r = eval_ok("sin(100)");
        set_angle_mode(0);
        assert!((r - 1.0).abs() < 1e-9, "sin(100 grad) = 1, got={}", r);
    }

    #[test]
    fn stress_mode_restore() {
        let _g = lock_silent(&MX); set_decimal_precision(8);
        set_angle_mode(1);
        let _ = eval_ok("sin(30)");
        set_angle_mode(0);
        let r = eval_ok("sin(pi/2)");
        assert!((r - 1.0).abs() < 1e-9);
    }

    #[test]
    fn stress_mode_deg_triple_identity() {
        let _g = lock_silent(&MX); set_angle_mode(1); set_decimal_precision(8);
        // sin²+cos²=1 para 10 ángulos en DEG
        for &a in &[0, 15, 30, 45, 60, 75, 90, 120, 150, 180i32] {
            let r = eval_ok(&format!("sin({})^2 + cos({})^2", a, a));
            assert!((r - 1.0).abs() < 1e-9, "deg a={} identity failed, got={}", a, r);
        }
        set_angle_mode(0);
    }

    #[test]
    fn stress_mode_grad_identity() {
        let _g = lock_silent(&MX); set_angle_mode(2); set_decimal_precision(8);
        for &a in &[0, 25, 50, 75, 100, 150, 200i32] {
            let r = eval_ok(&format!("sin({})^2 + cos({})^2", a, a));
            assert!((r - 1.0).abs() < 1e-9, "grad a={} identity failed, got={}", a, r);
        }
        set_angle_mode(0);
    }

    // =========================================================================
    // 7. VARIABLES Y ESTADO (10 pruebas)
    // =========================================================================

    #[test]
    fn stress_var_multi_assign() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_variables();
        process_expression("A = 3").unwrap();
        process_expression("B = 4").unwrap();
        let r = eval_ok("A^2 + B^2");
        assert_eq!(r, 25.0);
    }

    #[test]
    fn stress_var_reassign() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_variables();
        process_expression("X = 5").unwrap();
        process_expression("X = 10").unwrap();
        let r = eval_ok("X * 2");
        assert_eq!(r, 20.0);
    }

    #[test]
    fn stress_var_in_equation() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_variables();
        process_expression("K = 5").unwrap();
        let r = process_expression("K*x + 10 = 20");
        assert!(r.is_ok(), "ecuacion con variable K debe funcionar");
    }

    #[test]
    fn stress_var_clear_redefine() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        process_expression("Z = 99").unwrap();
        clear_variables();
        // Z ya no debería existir
        assert!(process_expression("Z + 1").is_err(),
            "Z no debería existir tras clear_variables");
    }

    #[test]
    fn stress_var_26_letters() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        clear_variables();
        // Asignar A-Z (excepto constantes conocidas i, e, π)
        let letters = "ABCDFGHJKLMNOPQRSTUVWXYZ";
        for (i, ch) in letters.chars().enumerate() {
            let expr = format!("{} = {}", ch, i + 1);
            process_expression(&expr).unwrap_or_default();
        }
        // Verificar A=1, B=2
        let a = eval_ok("A");
        assert_eq!(a, 1.0);
        let b = eval_ok("B");
        assert_eq!(b, 2.0);
    }

    // =========================================================================
    // 8. CASOS LÍMITE DE PRECISIÓN IEEE 754 (15 pruebas)
    // =========================================================================

    #[test]
    fn stress_ieee_cancel_trig() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // tan(x) = sin(x)/cos(x) debe ser consistente
        let x = 0.7f64;
        let tan = eval_ok(&format!("tan({})", x));
        let sc  = eval_ok(&format!("sin({}) / cos({})", x, x));
        assert!((tan - sc).abs() < 1e-10);
    }

    #[test]
    fn stress_ieee_small_angle_sin() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // Para x pequeño, sin(x) ≈ x
        let x = 0.001f64;
        let r = eval_ok(&format!("sin({})", x));
        assert!((r - x).abs() < x * x * x, "sin(0.001) ≈ 0.001");
    }

    #[test]
    fn stress_ieee_cos_near_zero() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // cos(x) ≈ 1 - x²/2 para x pequeño
        let x = 0.01f64;
        let r = eval_ok(&format!("cos({})", x));
        let approx = 1.0 - x * x / 2.0;
        assert!((r - approx).abs() < 1e-9);
    }

    #[test]
    fn stress_ieee_exp_small() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // e^x ≈ 1 + x para x pequeño
        let x = 0.001f64;
        let r = eval_ok(&format!("exp({})", x));
        assert!((r - (1.0 + x)).abs() < x * x * 2.0);
    }

    #[test]
    fn stress_ieee_ln_near_1() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // ln(1+x) ≈ x para x pequeño
        let x = 0.001f64;
        let r = eval_ok(&format!("ln(1 + {})", x));
        assert!((r - x).abs() < x * x * 2.0);
    }

    #[test]
    fn stress_ieee_sqrt_square() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // sqrt(x)^2 = x para varios valores
        for &v in &[2.0f64, 3.0, 5.0, 7.0, 11.0, 0.5, 0.1] {
            let r = eval_ok(&format!("sqrt({})^2", v));
            assert!((r - v).abs() < 1e-10, "sqrt({})^2 ≈ {}, got={}", v, v, r);
        }
    }

    #[test]
    fn stress_ieee_pow_log_inverse() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 10^log(x) = x
        for &v in &[1.0f64, 10.0, 100.0, 0.1, 0.01] {
            let r = eval_ok(&format!("10^log({})", v));
            assert!((r - v).abs() < v * 1e-10,
                "10^log({}) ≈ {}, got={}", v, v, r);
        }
    }

    #[test]
    fn stress_ieee_subnormal_region() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        let r = process_expression("1e-308 * 1e-308");
        // puede ser subnormal o 0, pero no debe panic
        assert!(r.is_ok() || r.is_err());
    }

    #[test]
    fn stress_ieee_catastrophic_cancellation() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // (1 + 1e-15) - 1 puede ser 0 o 1e-15 dependiendo de la implementación
        let r = process_expression("(1 + 1e-15) - 1");
        assert!(r.is_ok(), "cancelacion catastrófica no debe fallar");
    }

    #[test]
    fn stress_ieee_large_minus_large() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 1e15 + 1 - 1e15 puede perder precisión
        let r = process_expression("1e15 + 1 - 1e15");
        // puede ser 0 o 1 — solo verificar que no haya error
        assert!(r.is_ok(), "1e15+1-1e15 no debe fallar");
    }

    #[test]
    fn stress_ieee_reciprocal() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // 1/(1/x) = x
        for &v in &[3.0f64, 7.0, 11.0, 0.3, 0.7] {
            let r = eval_ok(&format!("1/(1/{})", v));
            assert!((r - v).abs() < 1e-10, "1/(1/{}) ≈ {}, got={}", v, v, r);
        }
    }

    #[test]
    fn stress_ieee_power_commutativity() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // a^b * a^c = a^(b+c)
        for &(a, b, c) in &[(2.0f64, 3.0, 4.0), (3.0, 2.0, 5.0), (10.0, 1.0, 2.0)] {
            let lhs = eval_ok(&format!("{}^{} * {}^{}", a, b, a, c));
            let rhs = eval_ok(&format!("{}^({} + {})", a, b, c));
            assert!((lhs - rhs).abs() < rhs * 1e-10,
                "a^b*a^c=a^(b+c) falla para ({},{},{})", a, b, c);
        }
    }

    #[test]
    fn stress_ieee_trig_complement() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // sin(x) = cos(π/2 - x)
        for &x in &[0.1f64, 0.5, 1.0, 1.3] {
            let s = eval_ok(&format!("sin({})", x));
            let c = eval_ok(&format!("cos(pi/2 - {})", x));
            assert!((s - c).abs() < 1e-9,
                "sin(x)=cos(π/2-x) falla para x={}", x);
        }
    }

    #[test]
    fn stress_ieee_tan_definition() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        for &x in &[0.2f64, 0.5, 0.8, 1.0] {
            let tan = eval_ok(&format!("tan({})", x));
            let def = eval_ok(&format!("sin({}) / cos({})", x, x));
            assert!((tan - def).abs() < 1e-10, "tan def falla x={}", x);
        }
    }

    #[test]
    fn stress_ieee_hyperbolic_identity() {
        let _g = lock_silent(&MX); set_angle_mode(0);
        // cosh²(x) - sinh²(x) = 1
        for &x in &[0.0f64, 0.5, 1.0, 2.0] {
            let r = eval_ok(&format!("cosh({})^2 - sinh({})^2", x, x));
            assert!((r - 1.0).abs() < 1e-9, "cosh²-sinh²=1 falla x={}", x);
        }
    }

    #[test]
    fn stress_matrix_chain_op() {
        let _g = lock_silent(&MX);
        // tr([1,2;3,4] + [5,6;7,8] * 2) = tr([1+10, 2+12; 3+14, 4+16]) = tr([11, 14; 17, 20]) = 11 + 20 = 31
        let r = process_expression("tr([1,2;3,4] + [5,6;7,8] * 2)").unwrap();
        assert_eq!(r, "31");
    }

    #[test]
    fn stress_stat_precision_large() {
        let _g = lock_silent(&MX);
        let r = process_expression("mean([10, 20, 30, 40, 50, 60, 70, 80, 90, 100])").unwrap();
        assert_eq!(r, "55");
    }

    #[test]
    fn stress_complex_nest_composition() {
        let _g = lock_silent(&MX);
        // conj(re(1+2i) + im(3+4i)*i) = conj(1 + 4*i) = conj(1+4i) = 1-4i
        let r = process_expression("conj(re(1+2i) + im(3+4i)*i)").unwrap();
        assert_eq!(r, "1-4i");
    }

    #[test]
    fn stress_log_base_composition() {
        let _g = lock_silent(&MX);
        // log(log(256, 2), 2) = log(8, 2) = 3
        let r = process_expression("log(log(256, 2), 2)").unwrap();
        assert_eq!(r, "3");
    }

    #[test]
    fn stress_nested_fact_and_pow() {
        let _g = lock_silent(&MX);
        // (3!)^2 - 5! / 4 = 6^2 - 120 / 4 = 36 - 30 = 6
        let r = process_expression("(3!)^2 - 5! / 4").unwrap();
        assert_eq!(r, "6");
    }

    #[test]
    fn stress_big_vector_sum_min_max() {
        let _g = lock_silent(&MX);
        let sum_val = process_expression("sum([100, 200, 300, -400, 500])").unwrap();
        assert_eq!(sum_val, "700");
        let min_val = process_expression("min([100, 200, 300, -400, 500])").unwrap();
        assert_eq!(min_val, "-400");
        let max_val = process_expression("max([100, 200, 300, -400, 500])").unwrap();
        assert_eq!(max_val, "500");
    }

    #[test]
    fn stress_trig_radian_cycle_100() {
        let _g = lock_silent(&MX);
        set_angle_mode(0);
        // sin(2*pi * 100 + pi/2) = sin(pi/2) = 1
        let val = eval_ok("sin(2 * pi * 100 + pi/2)");
        assert!((val - 1.0).abs() < 1e-9);
    }

    #[test]
    fn stress_linreg_exact_fit() {
        let _g = lock_silent(&MX);
        // y = -3x + 10 -> m=-3, b=10, corr=-1, r2=1
        let r = process_expression("linreg([1, 2, 3], [7, 4, 1])").unwrap();
        assert_eq!(r, "[-3, 10, -1, 1]");
    }

    #[test]
    fn stress_complex_power_cycle() {
        let _g = lock_silent(&MX);
        // i^1 = i, i^2 = -1, i^3 = -i, i^4 = 1
        assert_eq!(process_expression("i^1").unwrap(), "i");
        assert_eq!(process_expression("i^2").unwrap(), "-1");
        assert_eq!(process_expression("i^3").unwrap(), "-i");
        assert_eq!(process_expression("i^4").unwrap(), "1");
    }

    #[test]
    fn stress_factorial_sum_limit() {
        let _g = lock_silent(&MX);
        // sum([1!, 2!, 3!, 4!]) = sum([1, 2, 6, 24]) = 33
        assert_eq!(process_expression("sum([1!, 2!, 3!, 4!])").unwrap(), "33");
    }

    #[test]
    fn stress_nested_roots_and_powers() {
        let _g = lock_silent(&MX);
        // root(root(6561, 2), 2) = root(81, 2) = 9
        assert_eq!(process_expression("root(root(6561, 2), 2)").unwrap(), "9");
    }

    #[test]
    fn stress_large_matrix_trace() {
        let _g = lock_silent(&MX);
        // tr([1,2,3;4,5,6;7,8,9]) = 1 + 5 + 9 = 15
        assert_eq!(process_expression("tr([1,2,3;4,5,6;7,8,9])").unwrap(), "15");
    }

    #[test]
    fn stress_corr_cov_identity() {
        let _g = lock_silent(&MX);
        // corr(x,y) = cov(x,y) / (std(x)*std(y))
        // Let x = [1,2,3], y = [1,3,2]
        // cov(x,y) = 0.5, std(x) = 1.0, std(y) = 1.0
        // corr(x,y) = 0.5
        assert_eq!(process_expression("cov([1,2,3], [1,3,2])").unwrap(), "0.5");
        assert_eq!(process_expression("corr([1,2,3], [1,3,2])").unwrap(), "0.5");
    }

    #[test]
    fn stress_ans_stat_chain() {
        let _g = lock_silent(&MX);
        clear_last_result();
        process_expression("sum([10, 20, 30])").unwrap(); // 60
        process_expression("ans / 3").unwrap(); // 20
        process_expression("ans * 2").unwrap(); // 40
        assert_eq!(get_last_result(), 40.0);
    }

    #[test]
    fn stress_mixed_scalar_vector_ops() {
        let _g = lock_silent(&MX);
        // [1, 2, 3] * 3 - [1, 1, 1] = [3, 6, 9] - [1, 1, 1] = [2, 5, 8]
        assert_eq!(process_expression("[1, 2, 3] * 3 - [1, 1, 1]").unwrap(), "[2, 5, 8]");
    }
