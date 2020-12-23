use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum InstanceType {
    #[serde(rename = "CLASSCOMPONENT")]
    ClassComponent,
    #[serde(rename = "INTERFACECOMPONENT")]
    InterfaceComponent,
    #[serde(rename = "ANNOTATIONCOMPONENT")]
    AnnotationComponent,
    #[serde(rename = "METHODCOMPONENT")]
    MethodComponent,
    #[serde(rename = "MODULECOMPONENT")]
    ModuleComponent,
    #[serde(rename = "DIRECTORYCOMPONENT")]
    DirectoryComponent,
    #[serde(rename = "ANALYSISCOMPONENT")]
    AnalysisComponent,
    #[serde(rename = "FIELDCOMPONENT")]
    FieldComponent,
    #[serde(rename = "IMPORTCOMPONENT")]
    ImportComponent,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContainerStereotype {
    Fabricated,
    Controlled,
    Service,
    Response,
    Entity,
    Repository,
    Bean,
    Module,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContainerType {
    Class,
    Module,
    Interface,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct AnnotationValuePair<'a> {
    key: &'a str,
    value: &'a str,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum AccessorType {
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "protected")]
    Protected,
    #[serde(rename = "DEFAULT")]
    Default,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum LanguageFileType {
    Java,
    Cpp,
    Python,
    Go,
    // ...
    #[serde(rename = "N/A")]
    Unknown,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModuleStereotype {
    Fabricated,
    Controller,
    Service,
    Response,
    Entity,
    Repository, // The rest are for future expansion
                /*
                Bounded,
                Specification,
                ClosureOfOperations,
                Aggregation
                */
}
