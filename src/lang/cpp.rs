use crate::parse::AST;
use crate::prophet::*;

pub fn merge_modules(modules: Vec<ModuleComponent>) -> Vec<ModuleComponent> {
    let mut merged = vec![];
    for mut module in modules {
        if merged.len() == 0 {
            merged.push(module);
            continue;
        }

        let mergeable = merged
            .iter_mut()
            .find(|m| m.module_name == module.module_name);
        if let Some(mergeable) = mergeable {
            mergeable.classes.append(&mut module.classes);
            mergeable.interfaces.append(&mut module.interfaces);
            mergeable
                .component
                .methods
                .append(&mut module.component.methods);

            for class in mergeable.classes.iter_mut() {
                let methods: Vec<&mut MethodComponent> = mergeable
                    .component
                    .methods
                    .iter_mut()
                    .filter(|m| m.method_name.starts_with(&class.component.container_name))
                    .collect();
                for method in methods {
                    let class_method = class.component.methods.iter_mut().find(|m| {
                        m.method_name == method.method_name && m.parameters == method.parameters
                    });
                    if let Some(_class_method) = class_method {
                        // TODO: add the body from method into class_method
                    }
                }
            }

            mergeable.component.methods =
                mergeable
                    .component
                    .methods
                    .clone()
                    .into_iter()
                    .filter(|m| {
                        match mergeable.classes.iter_mut().find(|class| {
                            m.method_name.starts_with(&class.component.container_name)
                        }) {
                            Some(_) => false,
                            None => true,
                        }
                    })
                    .collect();
        }
    }
    merged
}

pub fn find_components(ast: AST, module_name: &str, path: &str) -> Vec<ComponentType> {
    match &*ast.r#type {
        "namespace_definition" => match transform_namespace_to_module(ast, path) {
            Some(module) => vec![ComponentType::ModuleComponent(module)],
            None => vec![],
        },
        "function_definition" => match transform_into_method(ast, module_name, path) {
            Some(method) => vec![ComponentType::MethodComponent(method)],
            None => vec![],
        },
        "class_specifier" => match transform_into_class(ast, module_name, path) {
            Some(class) => vec![ComponentType::ClassOrInterfaceComponent(class)],
            None => vec![],
        },
        "type_definition" => vec![], // could be typedef'd class
        _ => {
            let components: Vec<ComponentType> = ast
                .children
                .into_iter()
                .flat_map(|child| find_components(child, module_name, path))
                .collect();
            components
        }
    }
}

fn transform_namespace_to_module(ast: AST, path: &str) -> Option<ModuleComponent> {
    let name = ast
        .children
        .iter()
        .find(|child| child.r#type == "identifier")?
        .value
        .clone();

    let mut module = ModuleComponent::new(name.clone(), path.to_string());
    ast.children
        .into_iter()
        .flat_map(|child| find_components(child, &name, path))
        .for_each(|component| match component {
            ComponentType::ClassOrInterfaceComponent(component) => {
                match component.declaration_type {
                    ContainerType::Class => {
                        module.classes.push(component);
                    }
                    ContainerType::Interface => {
                        module.interfaces.push(component);
                    }
                    r#type => {
                        println!(
                            "got other label when it should have been class/ifc: {:#?}",
                            r#type
                        );
                    }
                }
            }
            ComponentType::MethodComponent(method) => {
                module.component.methods.push(method);
            }
            ComponentType::ModuleComponent(_module) => {
                unimplemented!();
            }
        });

    Some(module)
}

/// Transforms an AST with type label "function_definition" to a `MethodComponent`
fn transform_into_method(ast: AST, module_name: &str, path: &str) -> Option<MethodComponent> {
    // TODO: child type "compound_statement" for function block
    let ret = ast.children.iter().find(|child| match &*child.r#type {
        "primitive_type" | "scoped_type_identifier" | "type_identifier" => true,
        _ => false,
    });
    let ret_type = match ret {
        Some(ret) => type_ident(ret),
        None => "".to_string(),
    };

    let decl = ast
        .children
        .iter()
        .find(|child| child.r#type == "function_declarator")?;

    let identifier = decl.children.iter().find(|child| match &*child.r#type {
        "scoped_identifier" | "identifier" => true,
        _ => false,
    })?;
    let fn_ident = func_ident(identifier);

    let parameter_list = decl
        .children
        .iter()
        .find(|child| child.r#type == "parameter_list")?;
    let params = func_parameters(parameter_list, module_name, path);

    let method = MethodComponent {
        component: ComponentInfo {
            path: path.to_string(),
            package_name: module_name.to_string(),
            instance_name: fn_ident.clone(),
            instance_type: InstanceType::MethodComponent,
        },
        accessor: AccessorType::Default,
        method_name: fn_ident,
        return_type: ret_type,
        parameters: params,
        is_static: false,
        is_abstract: false,
        sub_methods: vec![],
        annotations: vec![],
        line_count: 0,
        line_begin: 0,
        line_end: 0,
    };

    Some(method)
}

/// Get the value for a type identifier
fn type_ident(ast: &AST) -> String {
    match &*ast.r#type {
        "primitive_type" | "type_identifier" => ast.value.clone(),
        "scoped_type_identifier" | "scoped_namespace_identifier" | "type_descriptor" => {
            let ret: String = ast
                .children
                .iter()
                .map(|child| match &*child.r#type {
                    "scoped_namespace_identifier" | "scoped_type_identifier" => type_ident(child),
                    _ => child.value.clone(),
                })
                .collect();
            ret
        }
        "template_type" => {
            let outer_type: String = ast
                .children
                .iter()
                .filter(|child| child.r#type != "template_argument_list")
                .map(|child| type_ident(&child))
                .collect();

            let type_args = ast
                .children
                .iter()
                .find(|child| child.r#type == "template_argument_list")
                .expect("No argument list for template");

            let inner_types = type_args
                .children
                .iter()
                .filter(|child| child.r#type == "type_descriptor")
                .map(|child| type_ident(&child))
                .fold(String::new(), |t1, t2| match &*t1 {
                    "" => t2,
                    _ => t1 + ", " + &t2,
                });

            format!("{}<{}>", outer_type, inner_types)
        }
        _ => ast.value.clone(),
    }
}

/// Get the value for a function identifier
fn func_ident(ast: &AST) -> String {
    match &*ast.r#type {
        "function_declarator" => {
            let ident = ast.children.iter().find(|child| match &*child.r#type {
                "scoped_identifier" | "identifier" => true,
                _ => false,
            });
            match ident {
                Some(ident) => func_ident(ident),
                None => "".to_string(),
            }
        }
        "scoped_identifier" => {
            let ident: String = ast
                .children
                .iter()
                .map(|child| match &*child.r#type {
                    "destructor_name" | "constructor_name" => func_ident(child),
                    _ => child.value.clone(),
                })
                .collect();
            ident
        }
        "destructor_name" | "constructor_name" => {
            let ident: String = ast
                .children
                .iter()
                .map(|child| child.value.clone())
                .collect();
            ident
        }
        "identifier" => ast.value.clone(),
        _ => "".to_string(),
    }
}

fn func_parameters(param_list: &AST, module_name: &str, path: &str) -> Vec<MethodParamComponent> {
    let params: Vec<MethodParamComponent> = param_list
        .children
        .iter()
        .filter(|child| child.r#type == "parameter_declaration")
        .map(|param_decl| func_parameter(param_decl, module_name, path))
        .filter_map(|param_decl| param_decl)
        .collect();

    params
}

fn func_parameter(param_decl: &AST, module_name: &str, path: &str) -> Option<MethodParamComponent> {
    let scoped_type_ident = param_decl
        .children
        .iter()
        .find(|child| match &*child.r#type {
            "scoped_type_identifier" | "primitive_type" | "type_identifier" => true,
            _ => false,
        })?;
    let mut param_type = type_ident(scoped_type_ident);

    let ident = param_decl
        .children
        .iter()
        .find(|child| match &*child.r#type {
            "pointer_declarator" | "reference_declarator" | "identifier" => true,
            _ => false,
        })?;

    let ident = match &*ident.r#type {
        "pointer_declarator" | "reference_declarator" => {
            ident
                .children
                .iter()
                .filter(|child| child.r#type != "identifier") // get either & or * type
                .for_each(|star| param_type.push_str(&star.value));
            ident
                .children
                .iter()
                .find(|child| child.r#type == "identifier")
                .map_or_else(|| "".to_string(), |identifier| identifier.value.clone())
        }
        "identifier" => ident.value.clone(),
        _ => "".to_string(),
    };

    let param = MethodParamComponent {
        component: ComponentInfo {
            path: path.to_string(),
            package_name: module_name.to_string(),
            instance_name: ident.clone(),
            instance_type: InstanceType::AnalysisComponent,
        },
        annotation: None,
        parameter_name: ident,
        parameter_type: param_type,
    };

    Some(param)
}

/// Transforms an AST with type label "class_specifier" to a `ClassOrInterfaceComponent`
fn transform_into_class(
    ast: AST,
    module_name: &str,
    path: &str,
) -> Option<ClassOrInterfaceComponent> {
    let class_name = ast
        .children
        .iter()
        .find(|child| child.r#type == "type_identifier")
        .map_or_else(|| "".into(), |t| t.value.clone());

    let fields_list = ast
        .children
        .iter()
        .find(|child| child.r#type == "field_declaration_list")?;

    let fields: Vec<FieldComponent> = fields_list
        .children
        .iter()
        .map(|child| class_field(child, AccessorType::Default))
        .filter_map(|field| field)
        .collect();

    Some(ClassOrInterfaceComponent {
        component: ContainerComponent {
            component: ComponentInfo {
                path: path.into(),
                package_name: module_name.into(),
                instance_name: class_name.clone(),
                instance_type: InstanceType::ClassComponent,
            },
            accessor: AccessorType::Default,
            stereotype: ContainerStereotype::Entity,
            methods: vec![],
            container_name: class_name,
            line_count: 0,
        },
        declaration_type: ContainerType::Class,
        annotations: vec![],
        stereotype: ContainerStereotype::Entity,
        constructors: None,
        field_components: Some(fields),
    })
}

fn class_field(ast: &AST, accessor: AccessorType) -> Option<FieldComponent> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_ident_primitive() {
        let prim = AST {
            children: vec![],
            span: None,
            r#type: "primitive_type".to_string(),
            value: "uint32_t".to_string(),
        };
        assert_eq!("uint32_t".to_string(), type_ident(&prim));
    }

    #[test]
    fn type_ident_scoped() {
        let t = AST {
            children: vec![
                AST {
                    children: vec![
                        AST {
                            children: vec![],
                            span: None,
                            r#type: "::".to_string(),
                            value: "::".to_string(),
                        },
                        AST {
                            children: vec![],
                            span: None,
                            r#type: "namespace_identifier".to_string(),
                            value: "thrift".to_string(),
                        },
                    ],
                    span: None,
                    r#type: "scoped_namespace_identifier".to_string(),
                    value: "".to_string(),
                },
                AST {
                    children: vec![],
                    span: None,
                    r#type: "::".to_string(),
                    value: "::".to_string(),
                },
                AST {
                    children: vec![],
                    span: None,
                    r#type: "namespace_identifier".to_string(),
                    value: "protocol".to_string(),
                },
            ],
            span: None,
            r#type: "scoped_namespace_identifier".to_string(),
            value: "".to_string(),
        };
        assert_eq!("::thrift::protocol".to_string(), type_ident(&t));
    }

    #[test]
    fn type_ident_generics() {
        let t = AST {
            children: vec![
                AST {
                    children: vec![
                        AST {
                            children: vec![],
                            span: None,
                            r#type: "namespace_identifier".to_string(),
                            value: "stdcxx".to_string(),
                        },
                        AST {
                            children: vec![],
                            span: None,
                            r#type: "::".to_string(),
                            value: "::".to_string(),
                        },
                        AST {
                            children: vec![],
                            span: None,
                            r#type: "type_identifier".to_string(),
                            value: "shared_ptr".to_string(),
                        },
                    ],
                    span: None,
                    r#type: "scoped_type_identifier".to_string(),
                    value: "".to_string(),
                },
                AST {
                    children: vec![AST {
                        children: vec![AST {
                            children: vec![
                                AST {
                                    children: vec![
                                        AST {
                                            children: vec![
                                                AST {
                                                    children: vec![],
                                                    span: None,
                                                    r#type: "::".to_string(),
                                                    value: "::".to_string(),
                                                },
                                                AST {
                                                    children: vec![],
                                                    span: None,
                                                    r#type: "namespace_identifier".to_string(),
                                                    value: "apache".to_string(),
                                                },
                                            ],
                                            span: None,
                                            r#type: "scoped_namespace_identifier".to_string(),
                                            value: "".to_string(),
                                        },
                                        AST {
                                            children: vec![],
                                            span: None,
                                            r#type: "::".to_string(),
                                            value: "::".to_string(),
                                        },
                                        AST {
                                            children: vec![],
                                            span: None,
                                            r#type: "namespace_identifier".to_string(),
                                            value: "thrift".to_string(),
                                        },
                                    ],
                                    span: None,
                                    r#type: "scoped_namespace_identifier".to_string(),
                                    value: "".to_string(),
                                },
                                AST {
                                    children: vec![],
                                    span: None,
                                    r#type: "::".to_string(),
                                    value: "::".to_string(),
                                },
                                AST {
                                    children: vec![],
                                    span: None,
                                    r#type: "type_identifier".to_string(),
                                    value: "TProcessor".to_string(),
                                },
                            ],
                            span: None,
                            r#type: "scoped_type_identifier".to_string(),
                            value: "".to_string(),
                        }],
                        span: None,
                        r#type: "type_descriptor".to_string(),
                        value: "".to_string(),
                    }],
                    span: None,
                    r#type: "template_argument_list".to_string(),
                    value: "".to_string(),
                },
            ],
            span: None,
            r#type: "template_type".to_string(),
            value: "".to_string(),
        };
        assert_eq!(
            "stdcxx::shared_ptr<::apache::thrift::TProcessor>".to_string(),
            type_ident(&t)
        );
    }

    #[test]
    fn func_ident_destructor() {
        let f = AST {
            children: vec![AST {
                children: vec![
                    AST {
                        children: vec![],
                        span: None,
                        r#type: "namespace_identifier".to_string(),
                        value: "CastInfoService_WriteCastInfo_args".to_string(),
                    },
                    AST {
                        children: vec![],
                        span: None,
                        r#type: "::".to_string(),
                        value: "::".to_string(),
                    },
                    AST {
                        children: vec![
                            AST {
                                children: vec![],
                                span: None,
                                r#type: "~".to_string(),
                                value: "~".to_string(),
                            },
                            AST {
                                children: vec![],
                                span: None,
                                r#type: "identifier".to_string(),
                                value: "CastInfoService_WriteCastInfo_args".to_string(),
                            },
                        ],
                        span: None,
                        r#type: "destructor_name".to_string(),
                        value: "".to_string(),
                    },
                ],
                span: None,
                r#type: "scoped_identifier".to_string(),
                value: "".to_string(),
            }],
            span: None,
            r#type: "function_declarator".to_string(),
            value: "".to_string(),
        };
        assert_eq!(
            "CastInfoService_WriteCastInfo_args::~CastInfoService_WriteCastInfo_args".to_string(),
            func_ident(&f)
        );
    }

    #[test]
    fn func_ident_regular() {
        let f = AST {
            children: vec![AST {
                children: vec![
                    AST {
                        children: vec![],
                        span: None,
                        r#type: "namespace_identifier".to_string(),
                        value: "CastInfoService_WriteCastInfo_args".to_string(),
                    },
                    AST {
                        children: vec![],
                        span: None,
                        r#type: "::".to_string(),
                        value: "::".to_string(),
                    },
                    AST {
                        children: vec![],
                        span: None,
                        r#type: "identifier".to_string(),
                        value: "read".to_string(),
                    },
                ],
                span: None,
                r#type: "scoped_identifier".to_string(),
                value: "".to_string(),
            }],
            span: None,
            r#type: "function_declarator".to_string(),
            value: "".to_string(),
        };
        assert_eq!(
            "CastInfoService_WriteCastInfo_args::read".to_string(),
            func_ident(&f)
        );
    }

    #[test]
    fn func_param_single() {
        let param = AST {
            children: vec![
                AST {
                    children: vec![
                        AST {
                            children: vec![
                                AST {
                                    children: vec![
                                        AST {
                                            children: vec![
                                                AST {
                                                    children: vec![],
                                                    span: None,
                                                    r#type: "::".to_string(),
                                                    value: "::".to_string(),
                                                },
                                                AST {
                                                    children: vec![],
                                                    span: None,
                                                    r#type: "namespace_identifier".to_string(),
                                                    value: "apache".to_string(),
                                                },
                                            ],
                                            span: None,
                                            r#type: "scoped_namespace_identifier".to_string(),
                                            value: "".to_string(),
                                        },
                                        AST {
                                            children: vec![],
                                            span: None,
                                            r#type: "::".to_string(),
                                            value: "::".to_string(),
                                        },
                                        AST {
                                            children: vec![],
                                            span: None,
                                            r#type: "namespace_identifier".to_string(),
                                            value: "thrift".to_string(),
                                        },
                                    ],
                                    span: None,
                                    r#type: "scoped_namespace_identifier".to_string(),
                                    value: "".to_string(),
                                },
                                AST {
                                    children: vec![],
                                    span: None,
                                    r#type: "::".to_string(),
                                    value: "::".to_string(),
                                },
                                AST {
                                    children: vec![],
                                    span: None,
                                    r#type: "namespace_identifier".to_string(),
                                    value: "protocol".to_string(),
                                },
                            ],
                            span: None,
                            r#type: "scoped_namespace_identifier".to_string(),
                            value: "".to_string(),
                        },
                        AST {
                            children: vec![],
                            span: None,
                            r#type: "::".to_string(),
                            value: "::".to_string(),
                        },
                        AST {
                            children: vec![],
                            span: None,
                            r#type: "type_identifier".to_string(),
                            value: "TProtocol".to_string(),
                        },
                    ],
                    span: None,
                    r#type: "scoped_type_identifier".to_string(),
                    value: "".to_string(),
                },
                AST {
                    children: vec![
                        AST {
                            children: vec![],
                            span: None,
                            r#type: "*".to_string(),
                            value: "*".to_string(),
                        },
                        AST {
                            children: vec![],
                            span: None,
                            r#type: "identifier".to_string(),
                            value: "name".to_string(),
                        },
                    ],
                    span: None,
                    r#type: "pointer_declarator".to_string(),
                    value: "".to_string(),
                },
            ],
            span: None,
            r#type: "parameter_declarator".to_string(),
            value: "".to_string(),
        };
        let actual_param = func_parameter(&param, "", "").unwrap();
        assert_eq!(
            "::apache::thrift::protocol::TProtocol*".to_string(),
            actual_param.parameter_type,
        );
        assert_eq!("name".to_string(), actual_param.parameter_name,);
    }
}
