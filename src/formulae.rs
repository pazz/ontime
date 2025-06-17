use z3::{ast::{Bool, Int}, Config, Context, Solver};
use z3::ast::Ast;

/// Represents a formula that can be evaluated for integer values by substituting
/// a variable and simplifying the result.
#[derive(Clone)]
pub struct Formula<'ctx> {
    formula: Bool<'ctx>,
    x: Int<'ctx>,
    ctx: &'ctx Context,
}

impl<'ctx> Formula<'ctx> {
    /// Constructs a new Formula from a Z3 boolean formula, variable, and context.
    pub fn new(formula: Bool<'ctx>, x: Int<'ctx>, ctx: &'ctx Context) -> Self {
        Formula { formula, x, ctx }
    }

    /// Evaluates the formula for a given integer value by substituting `x` and simplifying.
    /// If simplification does not yield a boolean, uses the Z3 solver.
    pub fn evaluate(&self, x_val: i64) -> bool {
        let val_ast = Int::from_i64(self.ctx, x_val);
        let substituted = self
            .formula
            .substitute::<Int>(&[(&self.x.clone().into(), &val_ast.into())]);
        let simplified = substituted.simplify();
        match simplified.as_bool() {
            Some(b) => b,
            None => {
                let solver = Solver::new(self.ctx);
                solver.assert(&substituted);
                solver.check() == z3::SatResult::Sat
            }
        }
    }

    /// Evaluates a slice of Formulae for a given integer value, returning a Vec of bools.
    /// Each formula is simplified, and if not possible, the solver is used.
    pub fn evaluate_many(formulae: &[Formula<'ctx>], x_val: i64) -> Vec<bool> {
        formulae.iter().map(|f| f.evaluate(x_val)).collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use z3::{Config, ast::Int};

    #[test]
    fn test_formula_evaluate() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);

        // x > 5
        let x = Int::new_const(&ctx, "x");
        let five = Int::from_i64(&ctx, 5);
        let formula_ast = x.gt(&five);

        let formula = Formula::new(formula_ast, x, &ctx);

        assert_eq!(formula.evaluate(4), false);
        assert_eq!(formula.evaluate(6), true);
    }
}
    #[test]
    fn test_formula_evaluate_many() {
        let cfg = Config::new();
        let ctx = Context::new(&cfg);

        // x > 5
        let x = Int::new_const(&ctx, "x");
        let five = Int::from_i64(&ctx, 5);
        let gt_five = x.gt(&five);

        // x % 2 == 0
        let two = Int::from_i64(&ctx, 2);
        let even = x.modulo(&two)._eq(&Int::from_i64(&ctx, 0));

        let formula1 = Formula::new(gt_five, x.clone(), &ctx);
        let formula2 = Formula::new(even, x.clone(), &ctx);

        let results = Formula::evaluate_many(&[formula1.clone(), formula2.clone()], 6);
        assert_eq!(results, vec![true, true]);

        let results = Formula::evaluate_many(&[formula1.clone(), formula2.clone()], 5);
        assert_eq!(results, vec![false, false]);
    }
