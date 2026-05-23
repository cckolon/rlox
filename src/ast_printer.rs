use crate::expr::{Expr, Literal};

pub fn print_ast(expr: &Expr) -> String {
    match expr {
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left_string = print_ast(left);
            let right_string = print_ast(right);
            let operator_lexeme = &operator.token.lexeme;
            format!("({left_string} {operator_lexeme} {right_string})")
        }
        Expr::Grouping { expression } => {
            let printed_expression = print_ast(expression);
            format!("({printed_expression})")
        }
        Expr::Literal { value } => match value {
            Literal::Number(val) => val.to_string(),
            Literal::String(val) => val.clone(),
            Literal::Bool(val) => val.to_string(),
            Literal::Nil => "nil".to_string(),
        },
        Expr::Unary { operator, right } => {
            let printed_expression = print_ast(right);
            let operator_lexeme = &operator.token.lexeme;
            format!("({operator_lexeme} {printed_expression})")
        }
    }
}
