pub(crate) mod lexer;
pub(crate) mod parser;
pub(crate) mod evaluator;
pub(crate) mod ast;
pub(crate) mod token;
pub(crate) mod calculator;
pub(crate) mod value;
pub use calculator::process_expression;
pub use calculator::process_expression_ext;
pub use calculator::CalculatorError;

// Suite de pruebas ampliadas (500+ casos)
#[cfg(test)] mod suite_static;
#[cfg(test)] mod suite_stress;
#[cfg(test)] mod suite_regression;

#[cfg(test)]
pub(crate) static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());