use rhai::{ASTNode, Engine as RhaiEngine};

pub fn get_defines_and_uses(engine: &RhaiEngine, script: &str) -> (Vec<String>, Vec<String>) {
    let ast = match engine.compile(script) {
        Ok(ast) => ast,
        Err(_) => return (vec![], vec![]),
    };

    let mut defines = Vec::new();
    let mut uses = Vec::new();
    ast.walk(&mut |nodes| {
        if let Some(node) = nodes.last() {
            match node {
                ASTNode::Stmt(rhai::Stmt::Var(x, ..)) => {
                    defines.push(x.0.name.to_string());
                }
                ASTNode::Expr(rhai::Expr::Variable(x, ..)) => {
                    let name = x.1.to_string();
                    if !defines.contains(&name) {
                        uses.push(name);
                    }
                }
                _ => {}
            }
        }
        true
    });

    uses.dedup();
    (defines, uses)
}
