use parser::{CSSParser, IParser};

use crate::utils::minify;

mod cssom;
mod dom;
mod parser;
mod style;
mod utils;

fn main() {
    let input = "
            div#id.hello {
                height: 100%;
                background: purple;
                color: #ffffff !important;
            }

            div.my-div,div.my-div-2 {
                width: 100px;
                height: 100%;
                background: blue;
                color: #ffffff;
            }

            html {
                background: green;
            }
        ";
    let parsed = CSSParser::new(input).parse();
    print!("{}", parsed);
    assert_eq!(minify(&parsed.to_string()), minify(input))
}
