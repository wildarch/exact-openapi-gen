#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! { }
}

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Reqwest(::reqwest::Error);
        Url(::reqwest::UrlError);
    }
}

extern crate xml;
use xml::reader::{EventReader, XmlEvent};
extern crate reqwest;
use reqwest::Url;


const SPEC_BASE_URL : &'static str = "https://start.exactonline.nl/docs/";
const SPEC_OVERVIEW : &'static str = "HlpRestAPIResources.aspx";
const SPEC_DETAIL : &'static str = "HlpRestAPIResourcesDetails.aspx";

pub fn fetch_endpoints() -> Result<Vec<Url>> {
    let overview_url = SPEC_BASE_URL.to_owned() + SPEC_OVERVIEW;
    let response = reqwest::get(&overview_url)?;
    let parser = EventReader::new(response);
    let mut results = Vec::new();
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                if name.local_name == "a" {
                    for attr in attributes {
                        if attr.name.local_name == "href" && attr.value.starts_with(SPEC_DETAIL) {
                            println!("{}", &attr);
                            let url = Url::parse(&(SPEC_BASE_URL.to_owned() + &attr.value))?;
                            results.push(url);
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

#[test]
fn it_fetches_endpoint_url() {
    const SPEC_URL_PREFIX: &'static str = "https://start.exactonline.nl/docs/HlpRestAPIResourcesDetails.aspx?name=";

    let urls = fetch_endpoints().expect("Successfully fetched endpoints");
    assert!(!urls.is_empty());
    for u in urls {
        if !u.as_str().starts_with(SPEC_URL_PREFIX) {
            println!("Invalid endpoint: {}", u);
            panic!();
        }
    }
}