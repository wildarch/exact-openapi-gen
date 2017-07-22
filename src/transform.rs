use openapi::{Contact, Info, License, Operation, Operations, Parameter, Response, Schema, Spec, ParameterOrRef, Security};
use acquisition::{EndpointDetails, EdmType};

use std::collections::BTreeMap;
use std::iter::FromIterator;
use errors::*;
use reqwest::Method;

fn build_paths<'a, T: Iterator<Item=&'a EndpointDetails>>(endpoints: T) -> Result<BTreeMap<String, Operations>> {
    let mut paths = BTreeMap::new();
    for endpoint in endpoints {
        if endpoint.methods.contains(&Method::Put) || endpoint.methods.contains(&Method::Delete) {
            let url = format!("{}(guid'{{id}}')", endpoint.uri);
            paths.insert(url, Operations {
                get: None,
                post: None,
                put: build_operation(Method::Put, endpoint),
                delete: build_operation(Method::Delete, endpoint),
                patch: None,
                head: None,
                options: None,
                parameters: None
            });
        }
        if endpoint.methods.contains(&Method::Get) || endpoint.methods.contains(&Method::Post) {
            paths.insert(endpoint.uri.clone(), Operations {
                get: build_operation(Method::Get, endpoint),
                post: build_operation(Method::Post, endpoint),
                put: None,
                delete: None,
                patch: None,
                head: None,
                options: None,
                parameters: None
            });
        }
    }
    Ok(paths)
}

fn build_operation<'a>(method: Method, details: &'a EndpointDetails) -> Option<Operation> {
    if details.methods.contains(&method) {
        let mut responses = BTreeMap::new();
        let success_status = match method {
            Method::Get => "200",
            Method::Post => "201",
            Method::Put => "204",
            Method::Delete => "200",
            _ => unreachable!()
        };
        let success_schema = if method == Method::Delete { None } else {
            Some(Schema {
                ref_path: Some(format!("#/definitions/{}Response", details.name)),
                description: None,
                schema_type: None,
                format: None,
                enum_values: None,
                required: None,
                items: None,
                properties: None
            })
        };
        responses.insert(success_status.to_owned(), Response {
            description: "Command successful".to_owned(),
            schema: success_schema
        });
        responses.insert("400".to_owned(), Response {
            description: "Bad request (syntax invalid)".to_owned(),
            schema: None
        });
        responses.insert("401".to_owned(), Response {
            description: "Unauthorized".to_owned(),
            schema: None
        });
        responses.insert("404".to_owned(), Response {
            description: "Not found".to_owned(),
            schema: None
        });
        responses.insert("500".to_owned(), Response {
            description: "Error".to_owned(),
            schema: Some(Schema {
                ref_path: Some("#/definitions/Error".to_owned()),
                description: None,
                schema_type: None,
                format: None,
                enum_values: None,
                required: None,
                items: None,
                properties: None,
            })
        });
        let mut parameters = Vec::new();
        if method == Method::Get {
            // Add $filter, $select
            parameters.push(ParameterOrRef::Ref {
                ref_path: "#/parameters/filter".to_owned()
            });
            parameters.push(ParameterOrRef::Ref {
                ref_path: "#/paramters/select".to_owned()
            });
        }
        if details.uri.contains("{division}") {
            parameters.push(ParameterOrRef::Ref {
                ref_path: "#/parameters/Division".to_owned()
            });
        }
        if method == Method::Post || method == Method::Put {
            let def_suffix = match method {
                Method::Post => "Post",
                Method::Put => "Put",
                // We checked this in the if guard
                _ => unreachable!()
            };
            parameters.push(ParameterOrRef::Parameter {
                name: "body".to_owned(),
                location: "body".to_owned(),
                required: Some(true),
                schema: Some(Schema {
                    ref_path: Some(format!("#/definitions/{}{}", details.name, def_suffix)),
                    description: None,
                    schema_type: None,
                    format: None,
                    enum_values: None,
                    required: None,
                    items: None,
                    properties: None,
                }),
                unique_items: None,
                param_type: None,
                format: None,
                description: None,
            })
        }
        if method == Method::Put || method == Method::Delete {
            parameters.push(ParameterOrRef::Parameter {
                name: "id".to_owned(),
                location: "path".to_owned(),
                required: Some(true),
                schema: None,
                unique_items: None,
                param_type: Some("string".to_owned()),
                format: None,
                description: Some("ID of the entity to modify/delete".to_owned()),
            });
        }

        Some(Operation {
            responses: responses,
            parameters: Some(parameters),
            summary: None,
            description: None,
            consumes: None,
            produces: None,
            schemes: None,
            tags: None,
            operation_id: None,
        })
    } else {
        // This operation is not implemented for the given endpoint
        None
    }
}

struct OpenApiType {
    type_: String,
    format: Option<String>,
}

impl OpenApiType {
    pub fn new<T: Into<String>>(type_: T, format: Option<T>) -> OpenApiType {
        OpenApiType {type_: type_.into(), format: format.and_then(|f| Some(f.into()))}
    }
}

impl From<EdmType> for OpenApiType {
    fn from(edm: EdmType) -> OpenApiType {
        let (t, f) = match edm {
            EdmType::Null => ("null", None),
            EdmType::Binary => ("string", Some("edm-binary")),
            EdmType::Boolean => ("boolean", None),
            EdmType::Byte => ("string", Some("edm-byte")),
            EdmType::DateTime => ("string", Some("edm-datetime")),
            EdmType::Decimal => ("string", Some("edm-decimal")),
            EdmType::Double => ("number", Some("double")),
            EdmType::Single => ("number", Some("float")),
            EdmType::Guid => ("string", Some("guid")),
            EdmType::Int16 => ("integer", Some("int16")),
            EdmType::Int32 => ("integer", Some("int32")),
            EdmType::Int64 => ("integer", Some("int64")),
            EdmType::SByte => ("integer", Some("edm-int8")),
            EdmType::String => ("string", None),
            EdmType::Time => ("string", Some("edm-time")),
            EdmType::DateTimeOffset => ("string", Some("edm-date-time-offset")),
        };
        OpenApiType::new(t, f)
    }
}

fn build_definition(method: Method, endpoint: &EndpointDetails) -> Schema {
    let properties = BTreeMap::from_iter(endpoint.properties.iter()
        .filter(|p| p.methods.contains(&method))
        .map(|p| {
            let openapi_type = OpenApiType::from(p.edm_type.clone());
            (p.name.clone(), Schema {
                ref_path: None,
                description: p.description.clone(),
                schema_type: Some(openapi_type.type_),
                format: openapi_type.format,
                enum_values: None,
                required: None,
                items: None,
                properties: None,
            })
        }));
    // If the method is Post of Put, all keys are required properties
    let required_properties = if method == Method::Post || method == Method::Put {
        Some(endpoint.properties.iter().filter(|p| p.key).map(|p| p.name.clone()).collect())
    } else {
        None
    };
    let schema = Schema {
        ref_path: None,
        description: None,
        schema_type: Some("object".to_owned()),
        format: None,
        enum_values: None,
        required: required_properties,
        items: None,
        properties: Some(properties),
    };
    if method == Method::Get {
        let mut data = BTreeMap::new();
        let mut d = BTreeMap::new();
        d.insert("results".to_owned(), Schema {
            ref_path: None,
            description: None,
            schema_type: Some("array".to_owned()),
            format: None,
            enum_values: None,
            required: None,
            items: Some(Box::new(schema)),
            properties: None,
        });
        data.insert("d".to_owned(), Schema {
            ref_path: None,
            description: None,
            schema_type: Some("object".to_owned()),
            format: None,
            enum_values: None,
            required: None,
            items: None,
            properties: Some(d)
        });
        let required = vec!["d".to_owned()];
        Schema {
            ref_path: None,
            description: None,
            schema_type: Some("object".to_owned()),
            format: None,
            enum_values: None,
            required: Some(required),
            items: None,
            properties: Some(data)
        }
    }
    else {
        schema
    }
}

fn build_definitions<'a, T: Iterator<Item=&'a EndpointDetails>>(endpoints: T) -> Result<BTreeMap<String, Schema>> {
    let mut definitions = BTreeMap::new();
    definitions.insert("Error".to_owned(), build_error_schema());
    for endpoint in endpoints {
        if endpoint.methods.contains(&Method::Get) || endpoint.methods.contains(&Method::Post) {
            definitions.insert(format!("{}Response", endpoint.name), build_definition(Method::Get, endpoint));
        }
        if endpoint.methods.contains(&Method::Post) {
            definitions.insert(format!("{}Post", endpoint.name), build_definition(Method::Post, endpoint));
        }
        if endpoint.methods.contains(&Method::Put) {
            definitions.insert(format!("{}Put", endpoint.name), build_definition(Method::Put, endpoint));
        }
    }
    Ok(definitions)
}

fn build_error_schema() -> Schema {
    let mut error_properties = BTreeMap::new();
    error_properties.insert("code".to_owned(), Schema {
        ref_path: None,
        description: None,
        schema_type: Some("string".to_owned()),
        format: None,
        enum_values: None,
        required: None,
        items: None,
        properties: None
    });
    let mut message_properties = BTreeMap::new();
    message_properties.insert("value".to_owned(), Schema {
        ref_path: None,
        description: Some("Error cause".to_owned()),
        schema_type: Some("string".to_owned()),
        format: None,
        enum_values: None,
        required: None,
        items: None,
        properties: None
    });
    error_properties.insert("message".to_owned(), Schema {
        ref_path: None,
        description: None,
        schema_type: Some("object".to_owned()),
        format: None,
        enum_values: None,
        required: None,
        items: None,
        properties: Some(message_properties)
    });
    let mut error_property = BTreeMap::new();
    error_property.insert("error".to_owned(), Schema {
        ref_path: None,
        description: None,
        schema_type: Some("object".to_owned()),
        format: None,
        enum_values: None,
        required: None,
        items: None,
        properties: Some(error_properties)
    });
    Schema {
        ref_path: None,
        description: None,
        schema_type: Some("object".to_owned()),
        format: None,
        enum_values: None,
        required: None,
        items: None,
        properties: Some(error_property)
    }
}

fn build_parameters<'a, T: Iterator<Item=&'a EndpointDetails>>(endpoints: T) -> Result<BTreeMap<String, Parameter>> {
    let mut parameters = BTreeMap::new();
    parameters.insert("Division".to_owned(), Parameter {
        name: "division".to_owned(),
        location: "path".to_owned(),
        required: Some(true),
        schema: None,
        unique_items: None,
        param_type: Some("integer".to_owned()),
        format: Some("int32".to_owned()),
        description: None
    });
    parameters.insert("filter".to_owned(), Parameter {
        name: "$filter".to_owned(),
        location: "query".to_owned(),
        required: Some(false),
        schema: None,
        unique_items: None,
        param_type: Some("string".to_owned()),
        format: Some("$filter".to_owned()),
        description: None,
    });
    parameters.insert("select".to_owned(), Parameter {
        name: "$select".to_owned(),
        location: "query".to_owned(),
        required: Some(false),
        schema: None,
        unique_items: None,
        param_type: Some("string".to_owned()),
        format: Some("$select".to_owned()),
        description: None,
    });
    Ok(parameters)
}

fn build_security_definitions() -> BTreeMap<String, Security> {
    let mut security_definitions = BTreeMap::new();
    security_definitions.insert("ApiKey".to_owned(), Security::ApiKey {
        name: "Authorization".to_owned(),
        location: "header".to_owned(),
    });
    security_definitions
}

fn build_security_requirements() -> Vec<BTreeMap<String, Vec<String>>> {
    let mut requirement = BTreeMap::new();
    requirement.insert("ApiKey".to_owned(), Vec::default());
    vec![requirement]
}

pub fn build_spec(endpoints: Vec<EndpointDetails>) -> Result<Spec> {
    Ok(Spec {
        swagger: "2.0".to_owned(),
        info: Info {
            title: Some("Exact Online REST API".to_owned()),
            description: Some("Autogenerated using exact-openapi-gen".to_owned()),
            terms_of_service: None,
            contact: Some(Contact {
                name: Some("Daan de Graaf".to_owned()),
                url: Some("https://github.com/wildarch".to_owned()),
                email: Some("daandegraaf9@gmail.com".to_owned()),
            }),
            license: Some(License {
                name: Some("MIT".to_owned()),
                url: None,
            }),
            version: Some(String::from(env!("CARGO_PKG_VERSION"))),
        },
        host: Some("start.exactonline.nl".to_owned()),
        base_path: Some("/".to_owned()),
        schemes: Some(["https".to_owned()].to_vec()),
        consumes: Some(["application/json".to_owned()].to_vec()),
        produces: Some(["application/json".to_owned()].to_vec()),
        tags: None,
        paths: build_paths(endpoints.iter())?,
        definitions: Some(build_definitions(endpoints.iter())?),
        parameters: Some(build_parameters(endpoints.iter())?),
        responses: None,
        security_definitions: Some(build_security_definitions()),
        security: Some(build_security_requirements()),
    })
}