use std::collections::HashMap;

use crate::parse::AST;
use crate::prophet::*;

mod class_def;
use class_def::*;

pub fn find_components(ast: AST, path: &str) -> Vec<ComponentType> {
    find_components_internal(ast, String::new(), path)
}

fn find_components_internal(ast: AST, mut package: String, path: &str) -> Vec<ComponentType> {
    let mut components = vec![];

    for node in ast
        .find_all_children_by_type(&[
            "type_declaration",
            "func_declaration",
        ])
        .get_or_insert(vec![])
        .iter()
    {
        match &*node.r#type {
            //"function_declaration" => match transform_into_method()
            "type_declaration" => {
                for decl in node.find_all_children_by_type(&["type_spec"]).get_or_insert(vec![]).iter() {
                    match &*decl.r#type {
                        "type_spec" =>  {
                            parse_struct(&ast, &package, path);
                        },
                        _ => {
                            println!("{}", &decl.r#type);
                        },
                    }
                }
            }
            tag => todo!("Cannot identify provided tag {:#?}", tag),
        };
    }

    components
}

/*
pub fn transform_into_method(ast: AST, module_name: &str, path: &str) -> Option<MethodComponent> {
    let decl = match ast.find_child_by_type(&[
        ""
    ])
}
 */
