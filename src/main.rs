extern crate exact_openapi_gen;
extern crate reqwest;
extern crate openapi;

use std::fs::File;
use std::io::Write;

fn main() {
    let selected_endpoint_names = vec![
        "SystemSystemMe",
        "ProjectProjects", 
        "ManufacturingOperations", 
        "ManufacturingShopOrderRoutingStepPlans",
        "PayrollEmploymentContracts",
        "ManufacturingTimeTransactions"
    ];

    let urls = exact_openapi_gen::fetch_endpoint_urls().expect("Fetched endpoint urls");
    let endpoints = urls.into_iter()
        .filter(|url| {
            let url = String::from(url.as_str());
            selected_endpoint_names.iter().any(|selected| url.ends_with(selected))
        })
        .filter_map(|url| {
            println!("{}", &url);
            exact_openapi_gen::fetch_endpoint_details(url).ok()
        }).collect();

    let spec = exact_openapi_gen::build_spec(endpoints);
    let json = openapi::to_json(&spec.expect("Valid spec")).expect("Valid json spec");
    let mut file = File::create("api.json").expect("File opened");
    file.write_all(json.as_bytes()).expect("Successfully written to file");
}