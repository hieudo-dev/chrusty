use crate::parser::Css;
use crate::parser::Parser;

mod cssom;
mod dom;
mod parser;

fn main() {
    let mut a = Parser::<Css>::new(
        "
        div#id.hello {
            height: 100%;
            background: purple;
            color: #ffffff !important;
        }

        div.my-div {
            width: 100px;
            height: 100%;
            background: blue;
            color: #ffffff;
        }

        html {
            background: green;
        }
        ",
    );
    print!("{}", a.parse())
}
