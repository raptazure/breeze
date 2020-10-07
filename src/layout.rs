use std::fmt;

use crate::css::{Unit, Value};
use crate::style::{Display, StyledNode};

#[derive(Clone)]
pub struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    box_type: BoxType,
    pub styled_node: &'a StyledNode<'a>,
    pub children: Vec<LayoutBox<'a>>,
}
#[derive(Clone, Copy, Default)]
pub struct Dimensions {
    pub content: Rectangle,
    padding: EdgeSizes,
    pub border: EdgeSizes,
    margin: EdgeSizes,
    current: Rectangle,
}

#[derive(Clone, Copy, Default)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Default)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Clone)]
pub enum BoxType {
    Block,
    Inline,
    InlineBlock,
    Anonymous,
}

impl<'a> LayoutBox<'a> {
    pub fn new(box_type: BoxType, styled_node: &'a StyledNode) -> LayoutBox<'a> {
        LayoutBox {
            box_type,
            styled_node,
            dimensions: Default::default(),
            children: Vec::new(),
        }
    }

    fn layout(&mut self, b_box: Dimensions) {
        match self.box_type {
            BoxType::Block => self.layout_block(b_box),
            BoxType::Inline => self.layout_block(b_box),
            BoxType::InlineBlock => self.layout_inline_block(b_box),
            BoxType::Anonymous => {}
        }
    }

    fn layout_inline_block(&mut self, b_box: Dimensions) {
        self.calculate_inline_width(b_box);
        self.calculate_inline_position(b_box);
        self.layout_children();
        self.calculate_height();
    }

    fn calculate_inline_width(&mut self, b_box: Dimensions) {
        let s = self.styled_node;
        let d = &mut self.dimensions;

        d.content.width = get_absolute_num(s, b_box, "width").unwrap_or(0.0);
        d.margin.left = s.num_or("margin-left", 0.0);
        d.margin.right = s.num_or("margin-right", 0.0);
        d.padding.left = s.num_or("padding-left", 0.0);
        d.padding.right = s.num_or("padding-right", 0.0);
        d.border.left = s.num_or("border-left-width", 0.0);
        d.border.right = s.num_or("border-right-width", 0.0);
    }

    fn calculate_inline_position(&mut self, b_box: Dimensions) {
        let style = self.styled_node;
        let d = &mut self.dimensions;

        d.margin.top = style.num_or("margin-top", 0.0);
        d.margin.bottom = style.num_or("margin-bottom", 0.0);
        d.border.top = style.num_or("border-top-width", 0.0);
        d.border.bottom = style.num_or("border-bottom-width", 0.0);
        d.padding.top = style.num_or("padding-top", 0.0);
        d.padding.bottom = style.num_or("padding-bottom", 0.0);

        d.content.x =
            b_box.content.x + b_box.current.x + d.margin.left + d.border.left + d.padding.left;
        d.content.y =
            b_box.content.height + b_box.content.y + d.margin.top + d.border.top + d.padding.top;
    }

    fn layout_block(&mut self, b_box: Dimensions) {
        self.calculate_width(b_box);
        self.calculate_position(b_box);
        self.layout_children();
        self.calculate_height();
    }

    fn calculate_width(&mut self, b_box: Dimensions) {
        let style = self.styled_node;
        let d = &mut self.dimensions;

        let width = get_absolute_num(style, b_box, "width").unwrap_or(0.0);
        let margin_l = style.value("margin-left");
        let margin_r = style.value("margin-right");

        let margin_l_num = match margin_l {
            Some(m) => match **m {
                Value::Other(ref s) => s.parse().unwrap_or(0.0),
                _ => 0.0,
            },
            None => 0.0,
        };
        let margin_r_num = match margin_r {
            Some(m) => match **m {
                Value::Other(ref s) => s.parse().unwrap_or(0.0),
                _ => 0.0,
            },
            None => 0.0,
        };

        d.border.left = style.num_or("border-left-width", 0.0);
        d.border.right = style.num_or("border-right-width", 0.0);
        d.padding.left = style.num_or("padding-left", 0.0);
        d.padding.right = style.num_or("padding-right", 0.0);

        let total = width
            + margin_l_num
            + margin_r_num
            + d.border.left
            + d.border.right
            + d.padding.left
            + d.padding.right;

        let underflow = b_box.content.width - total;

        match (width, margin_l, margin_r) {
            (0.0, _, _) => {
                if underflow >= 0.0 {
                    d.content.width = underflow;
                    d.margin.right = margin_r_num;
                } else {
                    d.margin.right = margin_r_num + underflow;
                    d.content.width = width;
                }
                d.margin.left = margin_l_num;
            }
            (w, None, Some(_)) if w != 0.0 => {
                d.margin.left = underflow;
                d.margin.right = margin_r_num;
                d.content.width = w;
            }
            (w, Some(_), None) if w != 0.0 => {
                d.margin.right = underflow;
                d.margin.left = margin_l_num;
                d.content.width = w;
            }
            (w, None, None) if w != 0.0 => {
                d.margin.left = underflow / 2.0;
                d.margin.right = underflow / 2.0;
                d.content.width = w;
            }
            (_, _, _) => {
                d.margin.right = margin_r_num + underflow;
                d.margin.left = margin_l_num;
                d.content.width = width
            }
        }
    }

    fn calculate_position(&mut self, b_box: Dimensions) {
        let style = self.styled_node;
        let d = &mut self.dimensions;

        d.margin.top = style.num_or("margin-top", 0.0);
        d.margin.bottom = style.num_or("margin-bottom", 0.0);
        d.border.top = style.num_or("border-top-width", 0.0);
        d.border.bottom = style.num_or("border-bottom-width", 0.0);
        d.padding.top = style.num_or("padding-top", 0.0);
        d.padding.bottom = style.num_or("padding-bottom", 0.0);

        d.content.x = b_box.content.x + d.margin.left + d.border.left + d.padding.left;
        d.content.y =
            b_box.content.height + b_box.content.y + d.margin.top + d.border.top + d.padding.top;
    }

    fn calculate_height(&mut self) {
        self.styled_node.value("height").map_or((), |h| match **h {
            Value::Length(n, _) => self.dimensions.content.height = n,
            _ => {}
        })
    }

    fn layout_children(&mut self) {
        let d = &mut self.dimensions;
        let mut max_child_height = 0.0;

        let mut prevBoxType = BoxType::Block;

        for child in &mut self.children {
            match prevBoxType {
                BoxType::InlineBlock => match child.box_type {
                    BoxType::Block => {
                        d.content.height += max_child_height;
                        d.current.x = 0.0;
                    }
                    _ => {}
                },
                _ => {}
            }

            child.layout(*d);
            let new_height = child.dimensions.margin_box().height;

            if new_height > max_child_height {
                max_child_height = new_height;
            }

            match child.box_type {
                BoxType::Block => d.content.height += child.dimensions.margin_box().height,
                BoxType::InlineBlock => {
                    d.current.x += child.dimensions.margin_box().width;

                    if d.current.x > d.content.width {
                        d.content.height += max_child_height;
                        d.current.x = 0.0;
                        child.layout(*d);
                        d.current.x += child.dimensions.margin_box().width;
                    }
                }
                _ => {}
            }
            prevBoxType = child.box_type.clone();
        }
    }
}

impl<'a> fmt::Debug for LayoutBox<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "type:\n  {:?}\n{:?}\n", self.box_type, self.dimensions)
    }
}

impl Dimensions {
    fn padding_box(&self) -> Rectangle {
        self.content.expanded(self.padding)
    }

    pub fn border_box(&self) -> Rectangle {
        self.padding_box().expanded(self.border)
    }

    fn margin_box(&self) -> Rectangle {
        self.border_box().expanded(self.margin)
    }
}

impl fmt::Debug for Dimensions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "content:\n  {:?}\npadding:\n  {:?}\nborder:\n  {:?}\nmargin:\n  {:?}",
            self.content, self.padding, self.border, self.margin
        )
    }
}

impl Rectangle {
    fn expanded(&self, e: EdgeSizes) -> Rectangle {
        Rectangle {
            x: self.x - e.left,
            y: self.y - e.top,
            width: self.width + e.left + e.right,
            height: self.height + e.top + e.bottom,
        }
    }
}

impl fmt::Debug for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "x: {}, y: {}, w: {}, h: {}",
            self.x, self.y, self.width, self.height
        )
    }
}
impl fmt::Debug for EdgeSizes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "l: {} r: {} top: {} bot: {}",
            self.left, self.right, self.top, self.bottom
        )
    }
}

impl fmt::Debug for BoxType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_type = match *self {
            BoxType::Block => "block",
            BoxType::Inline => "inline",
            BoxType::InlineBlock => "inline-block",
            BoxType::Anonymous => "anonymous",
        };

        write!(f, "{}", display_type)
    }
}

fn get_absolute_num(s_node: &StyledNode, b_box: Dimensions, prop: &str) -> Option<f32> {
    match s_node.value(prop) {
        Some(ref v) => match ***v {
            Value::Length(l, ref u) => match *u {
                Unit::Px => Some(l),
                Unit::Pct => Some(l * b_box.content.width / 100.0),
                _ => panic!("Unimplemented css length unit"),
            },
            _ => None,
        },
        None => None,
    }
}

pub fn layout_tree<'a>(
    root: &'a StyledNode<'a>,
    mut containing_block: Dimensions,
) -> LayoutBox<'a> {
    containing_block.content.height = 0.0;

    let mut root_box = build_layout_tree(root);
    root_box.layout(containing_block);
    return root_box;
}

fn build_layout_tree<'a>(node: &'a StyledNode) -> LayoutBox<'a> {
    let mut layout_node = LayoutBox::new(
        match node.get_display() {
            Display::Block => BoxType::Block,
            Display::Inline => BoxType::Inline,
            Display::InlineBlock => BoxType::InlineBlock,
            Display::None => BoxType::Anonymous,
        },
        node,
    );

    for child in &node.children {
        match child.get_display() {
            Display::Block => layout_node.children.push(build_layout_tree(child)),
            Display::Inline => layout_node.children.push(build_layout_tree(child)),
            Display::InlineBlock => layout_node.children.push(build_layout_tree(child)),
            Display::None => {}
        }
    }
    layout_node
}

pub fn pretty_print<'a>(n: &'a LayoutBox, level: usize) {
    println!("{}{:?}\n", level, n);

    for child in n.children.iter() {
        pretty_print(&child, level + 1);
    }
}
