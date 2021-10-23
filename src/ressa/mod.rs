mod node_pattern;

pub use node_pattern::*;

mod explorer;
pub use explorer::*;

mod context;
pub use context::*;

mod pattern_parser;
pub use pattern_parser::*;

mod callback;
pub use callback::*;

mod index;
pub use index::*;

use crate::ModuleComponent;

/// Run the user-defined parsers, in the order they were defined, on our AST
pub fn run_ressa_parse(ast: &mut Vec<ModuleComponent>, ressas: Vec<NodePattern>) -> ContextData {
    let mut ctx = ParserContext::default();

    // Explore
    for mut ressa in ressas.into_iter() {
        for module in ast.iter_mut() {
            module.explore(&mut ressa, &mut ctx);
        }
    }

    // Clean and return context
    ctx.into()
}
