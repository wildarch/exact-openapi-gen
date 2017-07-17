extern crate exact_openapi_gen;

const SPEC_URL_PREFIX: &'static str = "https://start.exactonline.nl/docs/HlpRestAPIResourcesDetails.aspx?name=";

#[test]
fn it_fetches_endpoints() {
    let urls = exact_openapi_gen::fetch_endpoints().expect("Successfully fetched endpoints");
    assert!(!urls.is_empty());
    for u in urls {
        if !u.starts_with(SPEC_URL_PREFIX) {
            println!("Invalid endpoint: {}", u);
            panic!();
        }
    }
}