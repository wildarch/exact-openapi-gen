extern crate exact_openapi_gen;

fn main() {
    let endpoints = exact_openapi_gen::fetch_endpoints().unwrap();
    for e in endpoints {
        println!("{}", e);
    }
}