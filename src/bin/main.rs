use std::fs;

use genine::dom;

fn main() {
    let contents = fs::read_to_string("index.html").unwrap();

    dom::print_tree(dom::create_parse_tree(contents), 0);
}
