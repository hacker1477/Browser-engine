use crate::css::{Color,Declaration,Rule,Selector,Stylesheet,SimpleSelector,Unit,Value};

use std::iter::Peekable;
use std::str::Chars;

pub struct Cssparser<'a>{
    chars: Peekable<Chars<'a>>,
}

impl<'a> Cssparser<'a> {
    pub fn new(full_css: &str) -> Cssparser{
        Cssparser {
             chars: full_css.chars().peekable(),
        }
    }

    pub fn parse_stylesheet(&mut self) -> Stylesheet{
        let mut stylesheet = Stylesheet::default();

        while self.chars.peek().is_some() {
            let selector = self.parse_selectors();
            let style = self.parse_declarations();
            let rule = Rule::new(selector,style);
        
            stylesheet.rules.push(rule);
        }

        stylesheet
    }

    pub fn parse_selectors(&mut self) -> Vec<Selector>{
        let mut selectors = Vec::new();

        while self.chars.peek().map_or(false, |c| *c!= '{') {
            let selector = self.parse_selector();

            if selector != Selector::default(){
                selectors.push(selector);
            }

            self.consume_while(char::is_whitespace);
            if self.chars.peek().map_or(false, |c| *c == ','){
                self.chars.next();
            }
        }

        self.chars.next();
        selectors
    }

    pub fn parse_selector(&mut self) -> Selector{
        let mut sselector = SimpleSelector::default();
        let mut selector = Selector::default();

        self.consume_while(char::is_whitespace);

        sselector.tag_name = match self.chars.peek() {
            Some(&c) if Self::is_valid_start_ident(c) => Some(self.parse_identifiter()),
            _ => None,
        };

        let mut multiple_ids = false;
        while self.chars
            .peek()
            .map_or(false, |c| *c != ',' && *c != '{' && !(*c).is_whitespace()) 
        {
            match self.chars.peek() {
                Some(&c) if c == '#' => {
                    self.chars.next();
                    if sselector.id.is_some() || multiple_ids{
                        sselector.id = None;
                        multiple_ids = true;
                        self.parse_id();
                    }else {
                        sselector.id = self.parse_id();
                    }
                }
                Some(&c) if c == '.' => {
                    self.chars.next();
                    let class_name = self.parse_identifiter();

                    if class_name != String::from(""){
                        sselector.classes.push(class_name);
                    }
                }
                _ => {
                    self.consume_while(|c| c!= ',' && c != '{');
                }
            }    
        }

        if sselector != SimpleSelector::default(){
            selector.simple.push(sselector);
        }

        selector
    }

    pub fn parse_identifiter(&mut self) -> String{
        let mut ident = String::new();

        match self.chars.peek() {
            Some(&c) => if Self::is_valid_start_ident(c){
                ident.push_str(&self.consume_while(Self::is_valid_ident))
            },
            None => {}
        }

        ident.to_lowercase()
    }

    pub fn parse_id(&mut self) -> Option<String>{
        match &self.parse_identifiter()[ .. ] {
            "" => None,
            s @ _ => Some(s.to_string()),
        }
    }

    pub fn parse_declarations(&mut self) -> Vec<Declaration>{
        let mut declarations = Vec::<Declaration>::new();
        
        while self.chars.peek().map_or(false, |c| *c != '}') {
            self.consume_while(char::is_whitespace);

            let propety = self.consume_while(|x| x != ':').to_lowercase();

            self.chars.next();
            self.consume_while(char::is_whitespace);

            let value = self.consume_while(|x| x!=';' && x != '\n' && x!= '}').to_lowercase();

            let value_enum = match propety.as_ref() {
                "background-color" | "border-color" | "color" => {
                    Value::Color(Self::translate_color(&value))
                }
                "margin-right"|
                "margin-left"|
                "margin-bottom"|
                "margin-top"|
                "padding-right"|
                "padding-left"|
                "padding-bottom"|
                "padding-top" |
                "border-right-width"|
                "border-left-width"|
                "border-bottom-width"|
                "border-top-width" |
                "height"|
                "width" => Self::translate_lenght(&value),
                _ => Value::Other(value),
            };

            let declaration = Declaration::new(propety,value_enum);

            if self.chars.peek().map_or(false, |c| *c == ';'){
                declarations.push(declaration);
                self.chars.next();
            }else {
                self.consume_while(char::is_whitespace);
                if self.chars.peek().map_or(false, |c| *c == '}'){
                    declarations.push(declaration);
                }
            }
            self.consume_while(char::is_whitespace);
        }

        self.chars.next();
        declarations
    }

    fn consume_while<F>(&mut self,condition:F) -> String 
    where 
        F: Fn(char) -> bool
        {
        let mut result = String::new();
        while self.chars.peek().map_or(false, |c| condition(*c)){
            result.push(self.chars.next().unwrap());
        }
        result
    }

    fn translate_lenght(value: &str) -> Value{
        let mut num_str = String::new();
        let mut unit = String::new();
        let mut parsing_num = true;

        for c in value.chars(){
            if c.is_numeric() && parsing_num{
                num_str.push(c);
            }else {
                unit.push(c);
                parsing_num = false;
            }
        }

        let number = num_str.parse().unwrap_or(0.0);

        match unit.as_ref() {
            "em" => Value::Length(number, Unit::Em),
            "ex" => Value::Length(number, Unit::Ex),
            "rem" => Value::Length(number, Unit::Rem),
            "ch" => Value::Length(number, Unit::Ch),
            "vh" => Value::Length(number, Unit::Vh),
            "vw" => Value::Length(number, Unit::Vw),
            "vmin" => Value::Length(number, Unit::Vmin),
            "vmax" => Value::Length(number, Unit::Vmax),
            "px" => Value::Length(number, Unit::Px),
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

    fn translate_color(color: &str) -> Color{
        if color.starts_with("#"){
            if color.len() == 7{
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
            }else if color.len() == 4 {
                let red = match u8::from_str_radix(&color[1..2], 16) {
                    Ok(n) => n as f32 /15.0,
                    Err(_) => 0.0,
                };
                let green = match u8::from_str_radix(&color[2..3], 16) {
                    Ok(n) => n as f32 /15.0,
                    Err(_) => 0.0,
                };
                let blue = match u8::from_str_radix(&color[3..4], 16) {
                    Ok(n) => n as f32 /15.0,
                    Err(_) => 0.0,
                };
                return Color::new(red, green, blue, 1.0);
            }else {
                return Color::default();
            }
        }else if color.starts_with("rgb") {
            return Color::default();
        }else if color.starts_with("hsl") {
            return Color::default();
        }else {
            return match color {
                "black" => Color::new(0.0, 0.0, 0.0, 1.0),
                "silver" => Color::new(0.75, 0.75, 0.75, 1.0),
                "gray"| "grey" => Color::new(0.5, 0.5, 0.5, 1.0),
                "white" => Color::new(1.0, 1.0, 1.0, 1.0),
                "yellowgreen" => Color::new(0.6, 0.8, 0.2, 1.0),
                "red" => Color::new(1.0, 0.0, 0.0, 1.0),
                "blue" => Color::new(0.0, 0.0, 1.0, 1.0),
                "green" => Color::new(0.0, 1.0, 0.0, 1.0),
                "purple" => Color::new(0.5, 0.0, 0.5, 1.0),
                "orange" => Color::new(1.0, 0.5, 0.0, 1.0),
                "aliceblue" => Color::new(240.0,248.0,255.0,1.0),
                "antiquewhite" => Color::new(250.0,235.0,215.0,1.0),
                "aqua" => Color::new(0.0,255.0,255.0,1.0),
                "aquamarine" => Color::new(127.0,255.0,212.0,1.0),
                "azure" => Color::new(240.0,255.0,255.0,1.0),
                "beige" => Color::new(245.0,245.0,220.0,1.0),
                "bisque" => Color::new(255.0,228.0,196.0,1.0),
                "blanchedalmond" => Color::new(255.0,235.0,205.0,1.0),
                "blueviolet" => Color::new(138.0,43.0,226.0,1.0),
                "brown" => Color::new(165.0,42.0,42.0,1.0),
                "burlywood" => Color::new(222.0,184.0,135.0,1.0),
                "cadetblue" => Color::new(95.0,158.0,160.0,1.0),
                "chartreuse" => Color::new(127.0,255.0,0.0,1.0),
                "chocolate" => Color::new(210.0,105.0,30.0,1.0),
                "coral" => Color::new(255.0,127.0,80.0,1.0),
                "cornflowerblue" => Color::new(100.0,149.0,237.0,1.0),
                "cornsilk" => Color::new(255.0,248.0,220.0,1.0),
                "crimson" => Color::new(220.0,20.0,60.0,1.0),
                 "cyan" => Color::new(0.0,255.0,255.0,1.0),
                "darkblue" => Color::new(0.0,0.0,139.0,1.0),
                "darkcyan" => Color::new(0.0,139.0,139.0,1.0),
                "darkgoldenrod" => Color::new(184.0,134.0,11.0,1.0),
                "darkgray" => Color::new(169.0,169.0,169.0,1.0),
                "darkgreen" => Color::new(0.0,100.0,0.0,1.0),
                "darkgrey" => Color::new(169.0,169.0,169.0,1.0),
                "darkkhaki" => Color::new(189.0,183.0,107.0,1.0),
                "darkmagenta" => Color::new(139.0,0.0,139.0,1.0),
                "darkolivegreen" => Color::new(85.0,107.0,47.0,1.0),
                "darkorange" => Color::new(255.0,140.0,0.0,1.0),
                "darkorchid" => Color::new(153.0,50.0,204.0,1.0),
                "darkred" => Color::new(139.0,0.0,0.0,1.0),
                "darksalmon" => Color::new(233.0,150.0,122.0,1.0),
                "darkseagreen" => Color::new(143.0,188.0,143.0,1.0),
                "darkslateblue" => Color::new(72.0,61.0,139.0,1.0),
                "darkslategray" => Color::new(47.0,79.0,79.0,1.0),
                "darkslategrey" => Color::new(47.0,79.0,79.0,1.0),
                "darkturquoise" => Color::new(0.0,206.0,209.0,1.0),
                "darkviolet" => Color::new(148.0,0.0,211.0,1.0),
                "deeppink" => Color::new(255.0,20.0,147.0,1.0),
                "deepskyblue" => Color::new(0.0,191.0,255.0,1.0),
                "dimgray" => Color::new(105.0,105.0,105.0,1.0),
                "dimgrey" => Color::new(105.0,105.0,105.0,1.0),
                "dodgerblue" => Color::new(30.0,144.0,255.0,1.0),
                "firebrick" => Color::new(178.0,34.0,34.0,1.0),
                "floralwhite" => Color::new(255.0,250.0,240.0,1.0),
                "forestgreen" => Color::new(34.0,139.0,34.0,1.0),
                "fuchsia" => Color::new(255.0,0.0,255.0,1.0),
                "gainsboro" => Color::new(220.0,220.0,220.0,1.0),
                "ghostwhite" => Color::new(248.0,248.0,255.0,1.0),
                "gold" => Color::new(255.0,215.0,0.0,1.0),
                "goldenrod" => Color::new(218.0,165.0,32.0,1.0),
                "greenyellow" => Color::new(173.0,255.0,47.0,1.0),
                "honeydew" => Color::new(240.0,255.0,240.0,1.0),
                "hotpink" => Color::new(255.0,105.0,180.0,1.0),
                "indianred" => Color::new(205.0,92.0,92.0,1.0),
                "indigo" => Color::new(75.0,0.0,130.0,1.0),
                "ivory" => Color::new(255.0,255.0,240.0,1.0),
                "khaki" => Color::new(240.0,230.0,140.0,1.0),
                "lavender" => Color::new(230.0,230.0,250.0,1.0),
                "lavenderblush" => Color::new(255.0,240.0,245.0,1.0),
                "lawngreen" => Color::new(124.0,252.0,0.0,1.0),
                "lemonchiffon" => Color::new(255.0,250.0,205.0,1.0),
                "lightblue" => Color::new(173.0,216.0,230.0,1.0),
                "lightcoral" => Color::new(240.0,128.0,128.0,1.0),
                "lightcyan" => Color::new(224.0,255.0,255.0,1.0),
                "lightgoldenrodyellow" => Color::new(250.0,250.0,210.0,1.0),
                "lightgray" => Color::new(211.0,211.0,211.0,1.0),
                "lightgreen" => Color::new(144.0,238.0,144.0,1.0),
                "lightgrey" => Color::new(211.0,211.0,211.0,1.0),
                "lightpink" => Color::new(255.0,182.0,193.0,1.0),
                "lightsalmon" => Color::new(255.0,160.0,122.0,1.0),
                "lightseagreen" => Color::new(32.0,178.0,170.0,1.0),
                "lightskyblue" => Color::new(135.0,206.0,250.0,1.0),
                "lightslategray" => Color::new(119.0,136.0,153.0,1.0),
                "lightslategrey" => Color::new(119.0,136.0,153.0,1.0),
                "lightsteelblue" => Color::new(176.0,196.0,222.0,1.0),
                "lightyellow" => Color::new(255.0,255.0,224.0,1.0),
                "lime" => Color::new(0.0,255.0,0.0,1.0),
                "limegreen" => Color::new(50.0,205.0,50.0,1.0),
                "linen" => Color::new(250.0,240.0,230.0,1.0),
                "magenta" => Color::new(255.0,0.0,255.0,1.0),
                "maroon" => Color::new(128.0,0.0,0.0,1.0),
                "mediumaquamarine" => Color::new(102.0,205.0,170.0,1.0),
                "mediumblue" => Color::new(0.0,0.0,205.0,1.0),
                "mediumorchid" => Color::new(186.0,85.0,211.0,1.0),
                "mediumpurple" => Color::new(147.0,112.0,219.0,1.0),
                "mediumseagreen" => Color::new(60.0,179.0,113.0,1.0),
                "mediumslateblue" => Color::new(123.0,104.0,238.0,1.0),
                "mediumspringgreen" => Color::new(0.0,250.0,154.0,1.0),
                "mediumturquoise" => Color::new(72.0,209.0,204.0,1.0),
                "mediumvioletred" => Color::new(199.0,21.0,133.0,1.0),
                "midnightblue" => Color::new(25.0,25.0,112.0,1.0),
                "mintcream" => Color::new(245.0,255.0,250.0,1.0),
                "mistyrose" => Color::new(255.0,228.0,225.0,1.0),
                "moccasin" => Color::new(255.0,228.0,181.0,1.0),
                "navajowhite" => Color::new(255.0,222.0,173.0,1.0),
                "navy" => Color::new(0.0,0.0,128.0,1.0),
                "oldlace" => Color::new(253.0,245.0,230.0,1.0),
                "olive" => Color::new(128.0,128.0,0.0,1.0),
                "olivedrab" => Color::new(107.0,142.0,35.0,1.0),
                "orangered" => Color::new(255.0,69.0,0.0,1.0),
                "orchid" => Color::new(218.0,112.0,214.0,1.0),
                "palegoldenrod" => Color::new(238.0,232.0,170.0,1.0),
                "palegreen" => Color::new(152.0,251.0,152.0,1.0),
                "paleturquoise" => Color::new(175.0,238.0,238.0,1.0),
                "palevioletred" => Color::new(219.0,112.0,147.0,1.0),
                "papayawhip" => Color::new(255.0,239.0,213.0,1.0),
                "peachpuff" => Color::new(255.0,218.0,185.0,1.0),
                "peru" => Color::new(205.0,133.0,63.0,1.0),
                "pink" => Color::new(255.0,192.0,203.0,1.0),
                "plum" => Color::new(221.0,160.0,221.0,1.0),
                "powderblue" => Color::new(176.0,224.0,230.0,1.0),
                "rosybrown" => Color::new(188.0,143.0,143.0,1.0),
                "royalblue" => Color::new(65.0,105.0,225.0,1.0),
                "saddlebrown" => Color::new(139.0,69.0,19.0,1.0),
                "salmon" => Color::new(250.0,128.0,114.0,1.0),
                "sandybrown" => Color::new(244.0,164.0,96.0,1.0),
                "seagreen" => Color::new(46.0,139.0,87.0,1.0),
                "seashell" => Color::new(255.0,245.0,238.0,1.0),
                "sienna" => Color::new(160.0,82.0,45.0,1.0),
                "skyblue" => Color::new(135.0,206.0,235.0,1.0),
                "slateblue" => Color::new(106.0,90.0,205.0,1.0),
                "slategray" => Color::new(112.0,128.0,144.0,1.0),
                "slategrey" => Color::new(112.0,128.0,144.0,1.0),
                "snow" => Color::new(255.0,250.0,250.0,1.0),
                "springgreen" => Color::new(0.0,255.0,127.0,1.0),
                "steelblue" => Color::new(70.0,130.0,180.0,1.0),
                "tan" => Color::new(210.0,180.0,140.0,1.0),
                "teal" => Color::new(0.0,128.0,128.0,1.0),
                "thistle" => Color::new(216.0,191.0,216.0,1.0),
                "tomato" => Color::new(255.0,99.0,71.0,1.0),
                "turquoise" => Color::new(64.0,224.0,208.0,1.0),
                "violet" => Color::new(238.0,130.0,238.0,1.0),
                "wheat" => Color::new(245.0,222.0,179.0,1.0),
                "whitesmoke" => Color::new(245.0,245.0,245.0,1.0),
                "yellow" => Color::new(255.0,255.0,0.0,1.0),
                _ => Color::new(0.0, 0.0, 0.0, 1.0),
            };
        }
    }


    fn is_valid_ident(c:char) -> bool{
        Self::is_valid_start_ident(c) || c.is_digit(10) || c == '-'
    }

    fn is_valid_start_ident(c:char) -> bool{
        Self::is_letter(c) || Self::is_non_ascii(c) ||  c == '_'
    }

    fn is_letter(c:char) -> bool{
        Self::is_upper_letter(c) || Self::is_lower_letter(c)
    }

    fn is_upper_letter(c:char) -> bool{
        c >= 'A' && c <= 'Z'
    }

    fn is_lower_letter(c:char) -> bool{
        c >= 'a' && c <= 'z'
    }

    fn is_non_ascii(c:char) -> bool{
        c.is_ascii()
    }
}