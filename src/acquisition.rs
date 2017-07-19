use reqwest::{self, Url, IntoUrl, Method};
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Name, Class, And};
use errors::*;
use errors::ErrorKind::SpecParseError;

use std::io::Read;
use std::convert::{TryFrom, TryInto};

const SPEC_BASE_URL : &'static str = "https://start.exactonline.nl/docs/";
const SPEC_OVERVIEW : &'static str = "HlpRestAPIResources.aspx";
const SPEC_DETAIL : &'static str = "HlpRestAPIResourcesDetails.aspx";

fn fetch_document<T: IntoUrl>(url: T) -> Result<Document> {
    let mut response = reqwest::get(url)?;
    let mut body = String::new();
    response.read_to_string(&mut body)?;
    Ok(Document::from(body.as_str()))
}

pub fn fetch_endpoint_urls() -> Result<Vec<Url>> {
    let overview_url = Url::parse(&(SPEC_BASE_URL.to_owned() + SPEC_OVERVIEW))?;
    let document = fetch_document(overview_url)?;
    let mut urls: Vec<Url> = document
        .find(And(Name("a"), Attr("href", ())))
        .filter_map(|node| {
            let href = node.attr("href").unwrap();
            if href.starts_with(SPEC_DETAIL) {
                Url::parse(&(SPEC_BASE_URL.to_owned() + href)).ok()
            }
            else {
                None
            }
        }).collect();
    urls.dedup();
    Ok(urls)
}

#[test]
fn it_fetches_endpoint_urls() {
    const SPEC_URL_PREFIX: &'static str = "https://start.exactonline.nl/docs/HlpRestAPIResourcesDetails.aspx?name=";

    let urls = fetch_endpoint_urls().expect("Successfully fetched endpoints");
    assert!(!urls.is_empty());
    for u in urls {
        if !u.as_str().starts_with(SPEC_URL_PREFIX) {
            println!("Invalid endpoint: {}", u);
            panic!();
        }
    }
}

#[derive(Debug)]
pub struct EndpointDetails {
    pub name: String,
    pub uri: String,
    pub properties: Vec<Property>,
    pub failed_properties: Vec<Error>,
    pub methods: Vec<Method>,
}

#[derive(Clone, Debug)]
pub struct Property {
    pub name: String,
    pub edm_type: EdmType,
    pub description: Option<String>,
    pub key: bool,
    pub methods: Vec<Method>,
}

impl<'a> TryFrom<Node<'a>> for Property {
    type Error = Error;
    fn try_from(n: Node<'a>) -> Result<Property> {
        let input = n.find(Name("input")).next()
            .ok_or(SpecParseError(format!("could not find name and type of Property: {:?}", n)))?;
        let description = n.children().nth(n.children().count() - 2)
            .ok_or(SpecParseError("could not find property description".to_owned()))?
            .text().trim().to_owned();
        let description = if description.is_empty() {
            None
        } else {
            Some(description)
        };
        let mut methods = Vec::new();
        if n.find(Class("showget")).count() > 0 {
            methods.push(Method::Get);
        }
        if n.find(Class("showpost")).count() > 0 {
            methods.push(Method::Post);
        }
        if n.find(Class("showput")).count() > 0 {
            methods.push(Method::Put);
        }
        if n.find(Class("showdelete")).count() > 0 {
            methods.push(Method::Delete);
        }
        Ok(Property {
            name: input.attr("name")
                .ok_or(SpecParseError("could not find property name".to_owned()))?.to_owned(),
            edm_type: input.attr("data-type")
                .ok_or(SpecParseError("could not find property type".to_owned()))?.try_into()
                .chain_err(|| format!("While parsing property {:?}", input.attr("name")))?,
            description: description,
            key: input.attr("data-key") == Some("True"),
            methods: methods,
        })
    }
}

// As defined in http://www.odata.org/documentation/odata-version-2-0/overview/#AbstractTypeSystem
#[derive(Clone, Debug)]
pub enum EdmType {
    Null,
    Binary,
    Boolean,
    Byte,
    DateTime,
    Decimal,
    Double,
    Single,
    Guid,
    Int16,
    Int32,
    Int64,
    SByte,
    String,
    Time,
    DateTimeOffset
}

impl<T: AsRef<str>> TryFrom<T> for EdmType {
    type Error = Error;
    fn try_from(s: T) -> Result<EdmType> {
        match s.as_ref() {
            "Edm.Null" => Ok(EdmType::Null),
            "Edm.Binary" => Ok(EdmType::Binary),
            "Edm.Boolean" => Ok(EdmType::Boolean),
            "Edm.Byte" => Ok(EdmType::Byte),
            "Edm.DateTime" => Ok(EdmType::DateTime),
            "Edm.Decimal" => Ok(EdmType::Decimal),
            "Edm.Double" => Ok(EdmType::Double),
            "Edm.Single" => Ok(EdmType::Single),
            "Edm.Guid" => Ok(EdmType::Guid),
            "Edm.Int16" => Ok(EdmType::Int16),
            "Edm.Int32" => Ok(EdmType::Int32),
            "Edm.Int64" => Ok(EdmType::Int64),
            "Edm.SByte" => Ok(EdmType::SByte),
            "Edm.String" => Ok(EdmType::String),
            "Edm.Time" => Ok(EdmType::Time),
            "Edm.DateTimeOffset" => Ok(EdmType::DateTimeOffset),
            _ => Err(SpecParseError(format!("Unknown type: {}", s.as_ref())).into())
        }
    }
} 


pub fn fetch_endpoint_details<T: IntoUrl>(url: T) -> Result<EndpointDetails> {
    let document = fetch_document(url)?;
    let name = document.find(Attr("id", "endpoint")).next()
        .ok_or(SpecParseError("name of endpoint not found".to_owned()))?
        .text();
    let uri = document.find(Attr("id", "serviceUri")).next()
        .ok_or(SpecParseError("uri of endpoint not found".to_owned()))?
        .text();
    let (properties, failed_properties) = document
        .find(Attr("id", "referencetable")).next()
        .ok_or(SpecParseError(format!("Endpoint {} - referencetable not found", name)))?
        .find(Name("tbody")).next()
        .ok_or(SpecParseError(format!("Endpoint {} - table body not found", name)))?
        // Skip the first row (header)
        .children().skip(1).filter(|c| c.name() == Some("tr"))
        .map(Property::try_from)
        .partition(|r| r.is_ok());
    let properties : Vec<Result<Property>> = properties;
    let properties: Vec<Property> = properties.into_iter().map(|p| p.unwrap()).collect();
    let failed_properties: Vec<Error> = failed_properties.into_iter().map(|p| p.err().unwrap()).collect();
    let methods = document.find(Attr("name", "supportedmethods"))
        .filter_map(|n| {
            match n.attr("value") {
                Some("GET") => Some(Method::Get),
                Some("POST") => Some(Method::Post),
                Some("PUT") => Some(Method::Put),
                Some("DELETE") => Some(Method::Delete),
                Some(m) => panic!("Unrecognized method in Endpoint {}: {}", name, m),
                None => None
            }
        }).collect();
    Ok(EndpointDetails {name, uri, properties, failed_properties, methods})
}

#[test]
fn it_fetches_endpoints_details() {
    let urls = fetch_endpoint_urls().expect("endpoints urls");

    let mut keyless_endpoints = Vec::new();
    let mut descriptionless_properties = Vec::new();
    let mut failed_endpoints = Vec::new();
    let mut failed_properties = Vec::new();

    for url in urls.clone() {
        match fetch_endpoint_details(url.clone()) {
            Ok(details) => {
                assert!(!details.name.is_empty(), "Could not find name of endpoint with details {:?}", details);
                assert!(!details.uri.is_empty(), "Could not find uri of endpoint with details {:?}", details);
                assert!(!details.methods.is_empty(), "Could not find methods of endpoint with details: {:?}", details);

                for err in details.failed_properties {
                    failed_properties.push((details.name.clone(), err));
                }
                let mut has_key = false;
                assert!(!details.properties.is_empty(), "Endpoint {} has no properties", details.name);
                for p in details.properties {
                    if p.key {
                        has_key = true;
                    }
                    assert!(!p.name.is_empty());
                    if p.description.is_none() {
                        descriptionless_properties.push((details.name.clone(), p.name));
                    }
                }
                if !has_key {
                    keyless_endpoints.push(details.name);
                }
            },
            Err(e) => failed_endpoints.push((url, e))
        } 
    }
    // Allow 1% of endpoints to have no primary key
    if (keyless_endpoints.len() as f32)/(urls.len() as f32) > 0.01 {
        println!("Warning: Many endpoints have no primary key. Endpoints without primary key:");
        for name in keyless_endpoints {
            println!("{}", name);
        }
        println!("Please verify this is correct according to the specification at {}{}", SPEC_BASE_URL, SPEC_OVERVIEW);
    }
    // Allow 1 property per endpoint without url
    if (descriptionless_properties.len() as f32)/(urls.len() as f32) > 1f32 {
        println!("Warning: Many properties have no description. Properties without description:");
        for (endpoint, property) in descriptionless_properties {
            println!("{}>{}", endpoint, property);
        }
        println!("Please verify this is correct according to the specification at {}{}", SPEC_BASE_URL, SPEC_OVERVIEW);
    }
    // Allow 2% of endpoints to fail
    if (failed_endpoints.len() as f32)/(urls.len() as f32) > 0.02 {
        println!("Warning: Many endpoints could not be parsed. Failed endpoints:");
        for (endpoint, error) in failed_endpoints {
            println!("{} failed with: {}", endpoint, error);
        }
    }
    // Allow one property per endpoint to fail parsing
    if (failed_properties.len() as f32)/(urls.len() as f32) > 1f32 {
        println!("Warning: Many properties could not be parsed. Failed properties:");
        for (endpoint, error) in failed_properties {
            println!("in {} failed with: {}", endpoint, error);
        }
    }
}
