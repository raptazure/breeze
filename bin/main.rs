extern crate breeze;

use breeze::{command, css, css_parser, dom, html_parser, layout, render, style};

use std::env;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let nodes = get_html();
    for n in nodes.iter() {
        dom::pretty_print(n, 0);
    }

    let ref root_node = nodes[0];

    let stylesheet = get_css();
    println!("{:?}", stylesheet);

    let style_tree_root = style::StyledNode::new(&root_node, &stylesheet);
    style::pretty_print(&style_tree_root, 0);

    let mut viewport = layout::Dimensions::default();
    viewport.content.width = 1024.0;
    viewport.content.height = 768.0;

    let layout_tree = layout::layout_tree(&style_tree_root, viewport);
    layout::pretty_print(&layout_tree, 0);

    let display_commands = command::build_display_commands(&layout_tree);
    render::render_loop(&display_commands);
}

fn get_html() -> Vec<dom::Node> {
    let mut path = env::current_dir().unwrap();
    path.push("example/example2.html");

    let mut file_reader = match File::open(&path) {
        Ok(f) => BufReader::new(f),
        Err(e) => panic!("file: {}, error: {}", path.display(), e),
    };

    let mut html_input = String::new();
    file_reader.read_to_string(&mut html_input).unwrap();

    let nodes = html_parser::HtmlParser::new(&html_input).parse_nodes();
    nodes
}

fn get_css() -> css::Stylesheet {
    let mut path = env::current_dir().unwrap();
    path.push("example/example2.css");

    let mut file_reader = match File::open(&path) {
        Ok(f) => BufReader::new(f),
        Err(e) => panic!("file: {}, error: {}", path.display(), e),
    };

    let mut css_input = String::new();
    file_reader.read_to_string(&mut css_input).unwrap();

    let stylesheet = css_parser::CssParser::new(&css_input).parse_stylesheet();
    stylesheet
}
