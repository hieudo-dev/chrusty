use std::collections::HashMap;

use dom::{new_element, new_text, TagType};

mod dom;

fn main() {
    let a = new_element(
        TagType::Div,
        HashMap::new(),
        vec![new_element(
            TagType::Div,
            HashMap::new(),
            vec![new_text("Hello, world!", vec![])],
        )],
    );
    print!("{}", a)
}
