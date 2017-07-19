extern crate exact_openapi_gen;
extern crate reqwest;
extern crate openapi;

fn main() {
    let url = reqwest::Url::parse("https://start.exactonline.nl/docs/HlpRestAPIResourcesDetails.aspx?name=AccountancyAccountOwners").unwrap();
    let details = exact_openapi_gen::fetch_endpoint_details(url).expect("parse details");
    println!("{:?}", &details);
    let endpoints = vec![details];

    let spec = exact_openapi_gen::build_spec(endpoints);
    let yaml = openapi::to_yaml(&spec.expect("Valid spec")).expect("Valid yaml spec");
    println!("Yaml: {}", yaml);
}