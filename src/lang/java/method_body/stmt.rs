use crate::java::method_body::log_unknown_tag;
use crate::java::method_body::parse_block;
use crate::java::method_body::parse_child_nodes;
use crate::java::modifier::find_modifier;
use crate::java::util::vartype::find_type;
use crate::ComponentInfo;
use crate::AST;
use crate::{ast::*, java::util::vartype::parse_type};

use super::{expr::parse_expr, node::parse_node};

/// File holding all Java statement parsing (e.g., while/for/trycatch)

/// Parse an AST section containing a variable declaration
pub(crate) fn parse_decl(ast: &AST, component: &ComponentInfo) -> DeclStmt {
    // Extract informtion about the variable
    let r#type = find_type(ast);
    let modifier = find_modifier(ast, &*component.path, &*component.package_name);

    // Determine the value it was set to
    let rhs = parse_child_nodes(ast, component);

    let mut decl = DeclStmt::new(vec![], vec![]);
    for var in rhs.iter() {
        let base;

        // Extract expression from the hierarchy
        if let Node::Stmt(Stmt::ExprStmt(ExprStmt { expr, .. })) = var {
            base = expr;
        } else if let Node::Expr(expr) = var {
            base = expr;
        } else {
            eprintln!("Unable to interpret as variable: {:#?}", var);
            continue;
        }

        // Parse variable
        match base {
            Expr::BinaryExpr(expr) => match expr.lhs.as_ref() {
                Expr::Ident(lhs) => {
                    decl.variables
                        .push(VarDecl::new(Some(r#type.clone()), lhs.clone()));
                    decl.expressions.push(expr.rhs.as_ref().clone());
                }
                unknown => eprintln!("Expected Ident got {:#?}", unknown),
            },
            Expr::Ident(id) => decl
                .variables
                .push(VarDecl::new(Some(r#type.clone()), id.clone())),
            unknown => {
                eprintln!("Expected BinaryExpr or Ident, got {:#?}", unknown);
            }
        }
    }

    // TODO: Use name
    for var_decl in decl.variables.iter_mut() {
        var_decl.is_final = Some(modifier.is_final);
        var_decl.is_static = Some(modifier.is_static);
        var_decl.var_type = Some(r#type.clone());
    }
    decl.into()
}

/// Parse an AST fragment with a try/catch. May be try-with-resources, or standard try/catch, with any
/// number of catch/multi-catch blocks, and/or a finally block.
pub(crate) fn try_catch(ast: &AST, component: &ComponentInfo) -> Option<Node> {
    let mut try_body = None;
    let mut catch_clauses = vec![];
    let mut finally_clause = None;
    // let mut resources = None;

    for comp in ast.children.iter() {
        match &*comp.r#type {
            "resource_specification" => {
                // let rss: Vec<Expr> = comp
                //     .children
                //     .iter()
                //     .filter(|resource| &*resource.r#type  == "resource")
                //     .map(|resource| parse_assignment(resource, component))
                //     .flat_map(|n| match n {
                //     })
                //     .collect();
                // resources = Some(DeclStmt::new(rss, expressions));
            }
            "block" => try_body = Some(parse_block(comp, component)),
            "catch_clause" => {
                let catch_comp = comp
                    .find_child_by_type(&["catch_formal_parameter"])
                    .expect("No catch variables declared!");

                // Modifiers
                let modifiers =
                    find_modifier(catch_comp, &*component.path, &*component.package_name);

                // Variables
                let caught_vars = {
                    let name = &catch_comp
                        .find_child_by_type(&["identifier"])
                        .expect("No name for caught variable!")
                        .value;
                    let types: Vec<String> = catch_comp
                        .find_child_by_type(&["catch_type"])
                        .expect("No type on catch block!")
                        .find_all_children_by_type(&["type_identifier"])
                        .expect("No type on catch block!")
                        .iter()
                        .map(|child| child.value.clone())
                        .collect();

                    types
                        .into_iter()
                        .map(|t| {
                            let mut decl = VarDecl::new(Some(t), Ident::new(name.clone()));
                            decl.is_final = Some(modifiers.is_final);
                            decl.is_static = Some(modifiers.is_static);
                            decl.annotation = modifiers.annotations.clone();
                            decl
                        })
                        .collect()
                };

                // Body
                let catch_body = parse_block(
                    comp.find_child_by_type(&["block"])
                        .expect("No block for the catch body!"),
                    component,
                );

                catch_clauses.push(CatchStmt::new(
                    DeclStmt::new(caught_vars, vec![]),
                    catch_body,
                ));
            }
            "finally_clause" => finally_clause = Some(parse_block(ast, component)),
            unknown => log_unknown_tag(unknown, "try/catch"),
        }
    }

    // Generated wrappers and return
    Some(Node::Stmt(
        TryCatchStmt::new(
            try_body.expect("Try/Catch with no body!"),
            catch_clauses,
            finally_clause,
        )
        .into(),
    ))
}

pub(crate) fn parse_for(ast: &AST, component: &ComponentInfo) -> Option<Node> {
    let mut clauses: Vec<Vec<&AST>> = vec![vec![], vec![], vec![], vec![]];
    let mut i = 0;

    // Coerce an Option<Node> to an Expr, if it can be
    let to_expr = |parts: &Vec<Node>| -> Vec<Expr> {
        parts
            .into_iter()
            .flat_map(|part| match part.clone() {
                Node::Expr(node) => Some(node),
                Node::Stmt(Stmt::ExprStmt(ExprStmt { expr, .. })) => Some(expr),
                _ => None,
            })
            .collect()
    };

    // Find all init, guard, and postcondition blocks
    for child in ast.children.iter() {
        match &*child.r#type {
            ";" | ")" => i = i + 1,
            "," | "for" | "(" => { /* Expected junk tags */ }
            _ => clauses[i].push(child),
        }
    }

    // Parse loop body (should be last)
    let body;
    let raw_body = clauses.pop()?;
    if raw_body.len() > 0 {
        body = parse_child_nodes(raw_body[0], component);
    } else {
        body = vec![];
    }

    // Parse for loop parts
    let parts: Vec<Option<Vec<Node>>> = clauses
        .iter()
        .map(|c| {
            if c.len() > 0 {
                Some(
                    c.iter()
                        .map(|c| parse_node(c, component))
                        .flat_map(|c| c)
                        .collect(),
                )
            } else {
                None
            }
        })
        .collect();

    // Parse initialization
    let init = parts[0].clone().map_or(vec![], |init_parts| {
        init_parts
            .into_iter()
            .flat_map(|p| match p {
                Node::Stmt(node) => Some(node),
                Node::Expr(node) => Some(Stmt::ExprStmt(ExprStmt::new(node))),
                _ => panic!("Not supported: block in for loop init"),
            })
            .collect()
    });

    // Parse guard condition
    let guard = parts[1]
        .clone()
        .map_or(None, |guard| Some(to_expr(&guard)[0].clone()));

    // Parse postcondition
    let post = parts[2].clone().map_or(vec![], |post| to_expr(&post));

    // Assemble
    Some(Stmt::ForStmt(ForStmt::new(init, guard, post, Block::new(body))).into())
}

pub(crate) fn parse_enhanced_for(ast: &AST, component: &ComponentInfo) -> Option<Node> {
    // Extract iterator
    let iter_type = parse_type(&ast.children[2]);
    let iter_var = DeclStmt::new(
        vec![VarDecl::new(
            Some(iter_type),
            Ident::new(ast.children[3].value.clone()),
        )],
        vec![],
    );
    let iter = parse_expr(&ast.children[5], component);

    // Extract body
    let body;
    if let Some(block) = ast.find_child_by_type(&["block"]) {
        body = parse_block(block, component);
    } else {
        body = Block::new(vec![]);
    }

    Some(Node::Stmt(
        ForRangeStmt::new(Box::new(Stmt::DeclStmt(iter_var)), iter, body).into(),
    ))
}

pub(crate) fn parse_labelled(ast: &AST, component: &ComponentInfo) -> Option<Node> {
    let label = LabelStmt::new(ast.children[0].value.clone());
    let body = parse_node(&ast.children[2], component);
    Some(Block::new(vec![Stmt::LabelStmt(label).into(), body?]).into())
}
