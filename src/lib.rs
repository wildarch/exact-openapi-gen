#![recursion_limit = "1024"]
extern crate xml;
extern crate reqwest;

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! { }
}

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Reqwest(::reqwest::Error);
    }
}

use errors::*;

use std::io::Read;
use xml::reader::{EventReader, XmlEvent};

const SPEC_BASE_URL : &'static str = "https://start.exactonline.nl/docs/";
const SPEC_OVERVIEW : &'static str = "HlpRestAPIResources.aspx";
const SPEC_DETAIL : &'static str = "HlpRestAPIResourcesDetails.aspx";

pub fn fetch_endpoints() -> Result<Vec<String>> {
    let url = SPEC_BASE_URL.to_owned() + SPEC_OVERVIEW;
    let response = reqwest::get(&url)?;
    let parser = EventReader::new(response);
    let mut results = Vec::new();
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                if name.local_name == "a" {
                    for attr in attributes {
                        if attr.name.local_name == "href" && attr.value.starts_with(SPEC_DETAIL) {
                            println!("{}", &attr);
                            results.push(SPEC_BASE_URL.to_owned() + &attr.value);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    Ok(results)
}