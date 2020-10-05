use crate::css::{Color, Declaration, Rule, Selector, SimpleSelector, Stylesheet, Unit, Value};

use std::iter::Peekable;
use std::str::Chars;

pub struct CssParser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> CssParser<'a> {
    pub fn new(full_css: &str) -> CssParser {
        CssParser {
            chars: full_css.chars().peekable(),
        }
    }

    pub fn parse_stylesheet(&mut self) -> Stylesheet {
        let mut stylesheet = Stylesheet::default();

        while self.chars.peek().is_some() {
            let selectors = self.parse_selectors();
            let styles = self.parse_declarations();
            let rule = Rule::new(selectors, styles);

            stylesheet.rules.push(rule);
        }

        stylesheet
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();

        while self.chars.peek().map_or(false, |c| *c != '{') {
            let selector = self.parse_selector();

            if selector != Selector::default() {
                selectors.push(selector);
            }

            self.consume_while(char::is_whitespace);
            if self.chars.peek().map_or(false, |c| *c == ',') {
                self.chars.next();
            }
        }

        self.chars.next();
        selectors
    }

    fn parse_selector(&mut self) -> Selector {
        let mut sselector = SimpleSelector::default();
        let mut selector = Selector::default();

        self.consume_while(char::is_whitespace);

        sselector.tag_name = match self.chars.peek() {
            Some(&c) if is_valid_start_ident(c) => Some(self.parse_identifier()),
            _ => None,
        };

        let mut multiple_ids = false;
        while self
            .chars
            .peek()
            .map_or(false, |c| *c != ',' && *c != '{' && !(*c).is_whitespace())
        {
            match self.chars.peek() {
                Some(&c) if c == '#' => {
                    self.chars.next();
                    if sselector.id.is_some() || multiple_ids {
                        sselector.id = None;
                        multiple_ids = true;
                        self.parse_id();
                    } else {
                        sselector.id = self.parse_id();
                    }
                }
                Some(&c) if c == '.' => {
                    self.chars.next();
                    let class_name = self.parse_identifier();

                    if class_name != String::from("") {
                        sselector.classes.push(class_name);
                    }
                }
                _ => {
                    self.consume_while(|c| c != ',' && c != '{');
                }
            }
        }

        if sselector != SimpleSelector::default() {
            selector.simple.push(sselector);
        }

        selector
    }

    fn parse_identifier(&mut self) -> String {
        let mut ident = String::new();

        match self.chars.peek() {
            Some(&c) => {
                if is_valid_start_ident(c) {
                    ident.push_str(&self.consume_while(is_valid_ident))
                }
            }
            None => {}
        }

        ident.to_lowercase()
    }

    fn parse_id(&mut self) -> Option<String> {
        match &self.parse_identifier()[..] {
            "" => None,
            s @ _ => Some(s.to_string()),
        }
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::<Declaration>::new();

        while self.chars.peek().map_or(false, |c| *c != '}') {
            self.consume_while(char::is_whitespace);

            let property = self.consume_while(|x| x != ':').to_lowercase();

            self.chars.next();
            self.consume_while(char::is_whitespace);

            let value = self
                .consume_while(|x| x != ';' && x != '\n' && x != '}')
                .to_lowercase();

            let value_enum = match property.as_ref() {
                "background-color" | "border-color" | "color" => {
                    Value::Color(translate_color(&value))
                }
                "margin-right"
                | "margin-bottom"
                | "margin-left"
                | "margin-top"
                | "padding-right"
                | "padding-bottom"
                | "padding-left"
                | "padding-top"
                | "border-right-width"
                | "border-bottom-width"
                | "border-left-width"
                | "border-top-width"
                | "height"
                | "width" => translate_length(&value),
                _ => Value::Other(value),
            };

            let declaration = Declaration::new(property, value_enum);

            if self.chars.peek().map_or(false, |c| *c == ';') {
                declarations.push(declaration);
                self.chars.next();
            } else {
                self.consume_while(char::is_whitespace);
                if self.chars.peek().map_or(false, |c| *c == '}') {
                    declarations.push(declaration);
                }
            }
            self.consume_while(char::is_whitespace);
        }

        self.chars.next();
        declarations
    }

    fn consume_while<F>(&mut self, condition: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while self.chars.peek().map_or(false, |c| condition(*c)) {
            result.push(self.chars.next().unwrap());
        }

        result
    }
}

fn translate_length(value: &str) -> Value {
    let mut num_str = String::new();
    let mut unit = String::new();
    let mut parsing_num = true;

    for c in value.chars() {
        if c.is_numeric() && parsing_num {
            num_str.push(c);
        } else {
            unit.push(c);
            parsing_num = false;
        }
    }

    let number = num_str.parse().unwrap_or(0.0);

    match unit.as_ref() {
        "em" => Value::Length(number, Unit::Em),
        "ex" => Value::Length(number, Unit::Ex),
        "ch" => Value::Length(number, Unit::Ch),
        "rem" => Value::Length(number, Unit::Rem),
        "vh" => Value::Length(number, Unit::Vh),
        "vw" => Value::Length(number, Unit::Vw),
        "vmin" => Value::Length(number, Unit::Vmin),
        "vmax" => Value::Length(number, Unit::Vmax),
        "px" | "" => Value::Length(number, Unit::Px),
        "mm" => Value::Length(number, Unit::Mm),
        "q" => Value::Length(number, Unit::Q),
        "cm" => Value::Length(number, Unit::Cm),
        "in" => Value::Length(number, Unit::In),
        "pt" => Value::Length(number, Unit::Pt),
        "pc" => Value::Length(number, Unit::Pc),
        "%" => Value::Length(number, Unit::Pct),
        _ => Value::Length(number, Unit::Px),
    }
}

fn translate_color(color: &str) -> Color {
    if color.starts_with("#") {
        if color.len() == 7 {
            let red = match u8::from_str_radix(&color[1..3], 16) {
                Ok(n) => n as f32 / 255.0,
                Err(_) => 0.0,
            };
            let green = match u8::from_str_radix(&color[3..5], 16) {
                Ok(n) => n as f32 / 255.0,
                Err(_) => 0.0,
            };
            let blue = match u8::from_str_radix(&color[5..7], 16) {
                Ok(n) => n as f32 / 255.0,
                Err(_) => 0.0,
            };
            return Color::new(red, green, blue, 1.0);
        } else if color.len() == 4 {
            let red = match u8::from_str_radix(&color[1..2], 16) {
                Ok(n) => n as f32 / 15.0,
                Err(_) => 0.0,
            };
            let green = match u8::from_str_radix(&color[2..3], 16) {
                Ok(n) => n as f32 / 15.0,
                Err(_) => 0.0,
            };
            let blue = match u8::from_str_radix(&color[3..4], 16) {
                Ok(n) => n as f32 / 15.0,
                Err(_) => 0.0,
            };
            return Color::new(red, green, blue, 1.0);
        } else {
            return Color::default();
        }
    } else if color.starts_with("rgb") {
        return Color::default();
    } else if color.starts_with("hsl") {
        return Color::default();
    } else {
        return match color {
            "black" => Color::new(0.0, 0.0, 0.0, 1.0),
            "silver" => Color::new(
                0.7529411764705882,
                0.7529411764705882,
                0.7529411764705882,
                1.0,
            ),
            "gray" | "grey" => Color::new(
                0.5019607843137255,
                0.5019607843137255,
                0.5019607843137255,
                1.0,
            ),
            "white" => Color::new(1.0, 1.0, 1.0, 1.0),
            "maroon" => Color::new(0.5019607843137255, 0.0, 0.0, 1.0),
            "red" => Color::new(1.0, 0.0, 0.0, 1.0),
            "purple" => Color::new(0.5019607843137255, 0.0, 0.5019607843137255, 1.0),
            "fuchsia" => Color::new(1.0, 0.0, 1.0, 1.0),
            "green" => Color::new(0.0, 0.5019607843137255, 0.0, 1.0),
            "lime" => Color::new(0.0, 1.0, 0.0, 1.0),
            "olive" => Color::new(0.5019607843137255, 0.5019607843137255, 0.0, 1.0),
            "yellow" => Color::new(1.0, 1.0, 0.0, 1.0),
            "navy" => Color::new(0.0, 0.0, 0.5019607843137255, 1.0),
            "blue" => Color::new(0.0, 0.0, 1.0, 1.0),
            "teal" => Color::new(0.0, 0.5019607843137255, 0.5019607843137255, 1.0),
            "aqua" => Color::new(0.0, 1.0, 1.0, 1.0),
            "orange" => Color::new(1.0, 0.6470588235294118, 0.0, 1.0),
            "aliceblue" => Color::new(0.9411764705882353, 0.9725490196078431, 1.0, 1.0),
            "antiquewhite" => Color::new(
                0.9803921568627451,
                0.9215686274509803,
                0.8431372549019608,
                1.0,
            ),
            "aquamarine" => Color::new(0.4980392156862745, 1.0, 0.8313725490196079, 1.0),
            "azure" => Color::new(0.9411764705882353, 1.0, 1.0, 1.0),
            "beige" => Color::new(
                0.9607843137254902,
                0.9607843137254902,
                0.8627450980392157,
                1.0,
            ),
            "bisque" => Color::new(1.0, 0.8941176470588236, 0.7686274509803922, 1.0),
            "blanchedalmond" => Color::new(1.0, 0.9215686274509803, 0.803921568627451, 1.0),
            "blueviolet" => Color::new(
                0.5411764705882353,
                0.16862745098039217,
                0.8862745098039215,
                1.0,
            ),
            "brown" => Color::new(
                0.6470588235294118,
                0.16470588235294117,
                0.16470588235294117,
                1.0,
            ),
            "burlywood" => Color::new(
                0.8705882352941177,
                0.7215686274509804,
                0.5294117647058824,
                1.0,
            ),
            "cadetblue" => Color::new(
                0.37254901960784315,
                0.6196078431372549,
                0.6274509803921569,
                1.0,
            ),
            "chartreuse" => Color::new(0.4980392156862745, 1.0, 0.0, 1.0),
            "chocolate" => Color::new(
                0.8235294117647058,
                0.4117647058823529,
                0.11764705882352941,
                1.0,
            ),
            "coral" => Color::new(1.0, 0.4980392156862745, 0.3137254901960784, 1.0),
            "cornflowerblue" => Color::new(
                0.39215686274509803,
                0.5843137254901961,
                0.9294117647058824,
                1.0,
            ),
            "cornsilk" => Color::new(1.0, 0.9725490196078431, 0.8627450980392157, 1.0),
            "crimson" => Color::new(
                0.8627450980392157,
                0.0784313725490196,
                0.23529411764705882,
                1.0,
            ),
            "darkblue" => Color::new(0.0, 0.0, 0.5450980392156862, 1.0),
            "darkcyan" => Color::new(0.0, 0.5450980392156862, 0.5450980392156862, 1.0),
            "darkgoldenrod" => Color::new(
                0.7215686274509804,
                0.5254901960784314,
                0.043137254901960784,
                1.0,
            ),
            "darkgray" | "darkgrey" => Color::new(
                0.6627450980392157,
                0.6627450980392157,
                0.6627450980392157,
                1.0,
            ),
            "darkgreen" => Color::new(0.0, 0.39215686274509803, 0.0, 1.0),
            "darkkhaki" => Color::new(
                0.7411764705882353,
                0.7176470588235294,
                0.4196078431372549,
                1.0,
            ),
            "darkmagenta" => Color::new(0.5450980392156862, 0.0, 0.5450980392156862, 1.0),
            "darkolivegreen" => Color::new(
                0.3333333333333333,
                0.4196078431372549,
                0.1843137254901961,
                1.0,
            ),
            "darkorange" => Color::new(1.0, 0.5490196078431373, 0.0, 1.0),
            "darkorchid" => Color::new(0.6, 0.19607843137254902, 0.8, 1.0),
            "darkred" => Color::new(0.5450980392156862, 0.0, 0.0, 1.0),
            "darksalmon" => Color::new(
                0.9137254901960784,
                0.5882352941176471,
                0.47843137254901963,
                1.0,
            ),
            "darkseagreen" => Color::new(
                0.5607843137254902,
                0.7372549019607844,
                0.5607843137254902,
                1.0,
            ),
            "darkslateblue" => Color::new(
                0.2823529411764706,
                0.23921568627450981,
                0.5450980392156862,
                1.0,
            ),
            "darkslategray" | "darkslategrey" => Color::new(
                0.1843137254901961,
                0.30980392156862746,
                0.30980392156862746,
                1.0,
            ),
            "darkturquoise" => Color::new(0.0, 0.807843137254902, 0.8196078431372549, 1.0),
            "darkviolet" => Color::new(0.5803921568627451, 0.0, 0.8274509803921568, 1.0),
            "deeppink" => Color::new(1.0, 0.0784313725490196, 0.5764705882352941, 1.0),
            "deepskyblue" => Color::new(0.0, 0.7490196078431373, 1.0, 1.0),
            "dimgray" | "dimgrey" => Color::new(
                0.4117647058823529,
                0.4117647058823529,
                0.4117647058823529,
                1.0,
            ),
            "dodgerblue" => Color::new(0.11764705882352941, 0.5647058823529412, 1.0, 1.0),
            "firebrick" => Color::new(
                0.6980392156862745,
                0.13333333333333333,
                0.13333333333333333,
                1.0,
            ),
            "floralwhite" => Color::new(1.0, 0.9803921568627451, 0.9411764705882353, 1.0),
            "forestgreen" => Color::new(
                0.13333333333333333,
                0.5450980392156862,
                0.13333333333333333,
                1.0,
            ),
            "gainsboro" => Color::new(
                0.8627450980392157,
                0.8627450980392157,
                0.8627450980392157,
                1.0,
            ),
            "ghostwhite" => Color::new(0.9725490196078431, 0.9725490196078431, 1.0, 1.0),
            "gold" => Color::new(1.0, 0.8431372549019608, 0.0, 1.0),
            "goldenrod" => Color::new(
                0.8549019607843137,
                0.6470588235294118,
                0.12549019607843137,
                1.0,
            ),
            "greenyellow" => Color::new(0.6784313725490196, 1.0, 0.1843137254901961, 1.0),
            "honeydew" => Color::new(0.9411764705882353, 1.0, 0.9411764705882353, 1.0),
            "hotpink" => Color::new(1.0, 0.4117647058823529, 0.7058823529411765, 1.0),
            "indianred" => Color::new(
                0.803921568627451,
                0.3607843137254902,
                0.3607843137254902,
                1.0,
            ),
            "indigo" => Color::new(0.29411764705882354, 0.0, 0.5098039215686274, 1.0),
            "ivory" => Color::new(1.0, 1.0, 0.9411764705882353, 1.0),
            "khaki" => Color::new(
                0.9411764705882353,
                0.9019607843137255,
                0.5490196078431373,
                1.0,
            ),
            "lavender" => Color::new(
                0.9019607843137255,
                0.9019607843137255,
                0.9803921568627451,
                1.0,
            ),
            "lavenderblush" => Color::new(1.0, 0.9411764705882353, 0.9607843137254902, 1.0),
            "lawngreen" => Color::new(0.48627450980392156, 0.9882352941176471, 0.0, 1.0),
            "lemonchiffon" => Color::new(1.0, 0.9803921568627451, 0.803921568627451, 1.0),
            "lightblue" => Color::new(
                0.6784313725490196,
                0.8470588235294118,
                0.9019607843137255,
                1.0,
            ),
            "lightcoral" => Color::new(
                0.9411764705882353,
                0.5019607843137255,
                0.5019607843137255,
                1.0,
            ),
            "lightcyan" => Color::new(0.8784313725490196, 1.0, 1.0, 1.0),
            "lightgoldenrodyellow" => Color::new(
                0.9803921568627451,
                0.9803921568627451,
                0.8235294117647058,
                1.0,
            ),
            "lightgray" | "lightgrey" => Color::new(
                0.8274509803921568,
                0.8274509803921568,
                0.8274509803921568,
                1.0,
            ),
            "lightgreen" => Color::new(
                0.5647058823529412,
                0.9333333333333333,
                0.5647058823529412,
                1.0,
            ),
            "lightpink" => Color::new(1.0, 0.7137254901960784, 0.7568627450980392, 1.0),
            "lightsalmon" => Color::new(1.0, 0.6274509803921569, 0.47843137254901963, 1.0),
            "lightseagreen" => Color::new(
                0.12549019607843137,
                0.6980392156862745,
                0.6666666666666666,
                1.0,
            ),
            "lightskyblue" => Color::new(
                0.5294117647058824,
                0.807843137254902,
                0.9803921568627451,
                1.0,
            ),
            "lightslategray" | "lightslategrey" => {
                Color::new(0.4666666666666667, 0.5333333333333333, 0.6, 1.0)
            }
            "lightsteelblue" => Color::new(
                0.6901960784313725,
                0.7686274509803922,
                0.8705882352941177,
                1.0,
            ),
            "lightyellow" => Color::new(1.0, 1.0, 0.8784313725490196, 1.0),
            "limegreen" => Color::new(
                0.19607843137254902,
                0.803921568627451,
                0.19607843137254902,
                1.0,
            ),
            "linen" => Color::new(
                0.9803921568627451,
                0.9411764705882353,
                0.9019607843137255,
                1.0,
            ),
            "mediumaquamarine" => Color::new(0.4, 0.803921568627451, 0.6666666666666666, 1.0),
            "mediumblue" => Color::new(0.0, 0.0, 0.803921568627451, 1.0),
            "mediumorchid" => Color::new(
                0.7294117647058823,
                0.3333333333333333,
                0.8274509803921568,
                1.0,
            ),
            "mediumpurple" => Color::new(
                0.5764705882352941,
                0.4392156862745098,
                0.8588235294117647,
                1.0,
            ),
            "mediumseagreen" => Color::new(
                0.23529411764705882,
                0.7019607843137254,
                0.44313725490196076,
                1.0,
            ),
            "mediumslateblue" => Color::new(
                0.4823529411764706,
                0.40784313725490196,
                0.9333333333333333,
                1.0,
            ),
            "mediumspringgreen" => Color::new(0.0, 0.9803921568627451, 0.6039215686274509, 1.0),
            "mediumturquoise" => Color::new(0.2823529411764706, 0.8196078431372549, 0.8, 1.0),
            "mediumvioletred" => Color::new(
                0.7803921568627451,
                0.08235294117647059,
                0.5215686274509804,
                1.0,
            ),
            "midnightblue" => Color::new(
                0.09803921568627451,
                0.09803921568627451,
                0.4392156862745098,
                1.0,
            ),
            "mintcream" => Color::new(0.9607843137254902, 1.0, 0.9803921568627451, 1.0),
            "mistyrose" => Color::new(1.0, 0.8941176470588236, 0.8823529411764706, 1.0),
            "moccasin" => Color::new(1.0, 0.8941176470588236, 0.7098039215686275, 1.0),
            "navajowhite" => Color::new(1.0, 0.8705882352941177, 0.6784313725490196, 1.0),
            "oldlace" => Color::new(
                0.9921568627450981,
                0.9607843137254902,
                0.9019607843137255,
                1.0,
            ),
            "olivedrab" => Color::new(
                0.4196078431372549,
                0.5568627450980392,
                0.13725490196078433,
                1.0,
            ),
            "orangered" => Color::new(1.0, 0.27058823529411763, 0.0, 1.0),
            "orchid" => Color::new(
                0.8549019607843137,
                0.4392156862745098,
                0.8392156862745098,
                1.0,
            ),
            "palegoldenrod" => Color::new(
                0.9333333333333333,
                0.9098039215686274,
                0.6666666666666666,
                1.0,
            ),
            "palegreen" => Color::new(0.596078431372549, 0.984313725490196, 0.596078431372549, 1.0),
            "paleturquoise" => Color::new(
                0.6862745098039216,
                0.9333333333333333,
                0.9333333333333333,
                1.0,
            ),
            "palevioletred" => Color::new(
                0.8588235294117647,
                0.4392156862745098,
                0.5764705882352941,
                1.0,
            ),
            "papayawhip" => Color::new(1.0, 0.9372549019607843, 0.8352941176470589, 1.0),
            "peachpuff" => Color::new(1.0, 0.8549019607843137, 0.7254901960784313, 1.0),
            "peru" => Color::new(
                0.803921568627451,
                0.5215686274509804,
                0.24705882352941178,
                1.0,
            ),
            "pink" => Color::new(1.0, 0.7529411764705882, 0.796078431372549, 1.0),
            "plum" => Color::new(
                0.8666666666666667,
                0.6274509803921569,
                0.8666666666666667,
                1.0,
            ),
            "powderblue" => Color::new(
                0.6901960784313725,
                0.8784313725490196,
                0.9019607843137255,
                1.0,
            ),
            "rosybrown" => Color::new(
                0.7372549019607844,
                0.5607843137254902,
                0.5607843137254902,
                1.0,
            ),
            "royalblue" => Color::new(
                0.2549019607843137,
                0.4117647058823529,
                0.8823529411764706,
                1.0,
            ),
            "saddlebrown" => Color::new(
                0.5450980392156862,
                0.27058823529411763,
                0.07450980392156863,
                1.0,
            ),
            "salmon" => Color::new(
                0.9803921568627451,
                0.5019607843137255,
                0.4470588235294118,
                1.0,
            ),
            "sandybrown" => Color::new(
                0.9568627450980393,
                0.6431372549019608,
                0.3764705882352941,
                1.0,
            ),
            "seagreen" => Color::new(
                0.1803921568627451,
                0.5450980392156862,
                0.3411764705882353,
                1.0,
            ),
            "seashell" => Color::new(1.0, 0.9607843137254902, 0.9333333333333333, 1.0),
            "sienna" => Color::new(
                0.6274509803921569,
                0.3215686274509804,
                0.17647058823529413,
                1.0,
            ),
            "skyblue" => Color::new(
                0.5294117647058824,
                0.807843137254902,
                0.9215686274509803,
                1.0,
            ),
            "slateblue" => Color::new(
                0.41568627450980394,
                0.35294117647058826,
                0.803921568627451,
                1.0,
            ),
            "slategray" | "slategrey" => Color::new(
                0.4392156862745098,
                0.5019607843137255,
                0.5647058823529412,
                1.0,
            ),
            "snow" => Color::new(1.0, 0.9803921568627451, 0.9803921568627451, 1.0),
            "springgreen" => Color::new(0.0, 1.0, 0.4980392156862745, 1.0),
            "steelblue" => Color::new(
                0.27450980392156865,
                0.5098039215686274,
                0.7058823529411765,
                1.0,
            ),
            "tan" => Color::new(
                0.8235294117647058,
                0.7058823529411765,
                0.5490196078431373,
                1.0,
            ),
            "thistle" => Color::new(
                0.8470588235294118,
                0.7490196078431373,
                0.8470588235294118,
                1.0,
            ),
            "tomato" => Color::new(1.0, 0.38823529411764707, 0.2784313725490196, 1.0),
            "turquoise" => Color::new(
                0.25098039215686274,
                0.8784313725490196,
                0.8156862745098039,
                1.0,
            ),
            "violet" => Color::new(
                0.9333333333333333,
                0.5098039215686274,
                0.9333333333333333,
                1.0,
            ),
            "wheat" => Color::new(
                0.9607843137254902,
                0.8705882352941177,
                0.7019607843137254,
                1.0,
            ),
            "whitesmoke" => Color::new(
                0.9607843137254902,
                0.9607843137254902,
                0.9607843137254902,
                1.0,
            ),
            "yellowgreen" => Color::new(
                0.6039215686274509,
                0.803921568627451,
                0.19607843137254902,
                1.0,
            ),
            "rebeccapurple" => Color::new(0.4, 0.2, 0.6, 1.0),
            _ => Color::new(0.0, 0.0, 0.0, 1.0),
        };
    }
}

fn is_valid_ident(c: char) -> bool {
    is_valid_start_ident(c) || c.is_digit(10) || c == '-'
}

fn is_valid_start_ident(c: char) -> bool {
    is_letter(c) || is_non_ascii(c) || c == '_'
}

fn is_letter(c: char) -> bool {
    is_upper_letter(c) || is_lower_letter(c)
}

fn is_upper_letter(c: char) -> bool {
    c >= 'A' && c <= 'Z'
}

fn is_lower_letter(c: char) -> bool {
    c >= 'a' && c <= 'z'
}

fn is_non_ascii(c: char) -> bool {
    c >= '\u{0080}'
}
