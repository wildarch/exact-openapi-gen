#![recursion_limit = "1024"]
#![feature(try_from)]

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Reqwest(::reqwest::Error);
            Url(::reqwest::UrlError);
        }

        errors {
            SpecParseError(t : String) {
                description("could not parse spec")
                display("could not parse spec: '{}'", t)
            }
        }
    }
}

extern crate select;
extern crate reqwest;
extern crate openapi;

mod acquisition;
pub use acquisition::*;

mod transform;
pub use transform::*;