pub mod tokenizer {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Attribute {
        name: String,
        value: String,
    }

    #[derive(Debug, Clone)]
    pub enum Token {
        Start,
        OpeningTag(String, Vec<Attribute>),
        ClosingTag(String),
        Text(String),
        End,
    }

    #[derive(PartialEq, Clone)]
    pub enum State {
        InTag,
        OutTag,
    }

    #[derive(PartialEq, Clone)]
    pub enum AttributeState {
        Name,
        Value,
    }

    fn parse_attributes(s: &str) -> Vec<Attribute> {
        let s = s.trim();

        let mut attr: Vec<Attribute> = vec![];
        let mut state: AttributeState = AttributeState::Name;
        let mut in_quotation = false;

        let mut attr_name = String::from("");
        let mut attr_value = String::from("");

        for c in s.chars() {
            match c {
                '=' => {
                    if in_quotation {
                        assert!(state == AttributeState::Value);
                        attr_value.push(c);
                    } else {
                        state = AttributeState::Value;
                    }
                }
                ' ' => {
                    if in_quotation {
                        assert!(state == AttributeState::Value);
                        attr_value.push(c);
                    } else {
                        assert!(state != AttributeState::Name);

                        // Add the new attribute into the vector
                        attr.push(Attribute {
                            name: attr_name,
                            value: attr_value,
                        });

                        attr_name = String::from("");
                        attr_value = String::from("");

                        state = AttributeState::Name;
                    }
                }
                '"' => {
                    assert!(state == AttributeState::Value);

                    if in_quotation {
                        in_quotation = false
                    } else {
                        in_quotation = true
                    }
                }
                _ => match state {
                    AttributeState::Name => {
                        attr_name.push(c);
                    }
                    AttributeState::Value => {
                        attr_value.push(c);
                    }
                },
            }
        }

        assert!(!in_quotation);

        if attr_name.chars().count() > 0 && attr_value.chars().count() > 0 {
            attr.push(Attribute {
                name: attr_name,
                value: attr_value,
            });
        }

        attr
    }

    pub fn tokenize(html: String) -> Vec<Token> {
        let mut pos = 0;
        let mut tokens: Vec<Token> = vec![];

        tokens.push(Token::Start);

        let mut word: String = String::from("");
        let mut state: State = State::OutTag;

        while pos < html.chars().count() {
            let c = html.chars().nth(pos).unwrap();

            let trimmed_word = word.trim();

            match c {
                '<' => {
                    if trimmed_word.chars().count() != 0 {
                        tokens.push(Token::Text(String::from(trimmed_word)));
                        word = String::from("");
                    }

                    state = State::InTag;
                }
                '>' => {
                    assert!(state == State::InTag);

                    if trimmed_word.chars().count() != 0 {
                        let (tag_name, attributes) =
                            trimmed_word.split_once(' ').unwrap_or((trimmed_word, ""));

                        let attr = parse_attributes(attributes);

                        if tag_name.starts_with("/") {
                            let tag_name: String = tag_name.chars().skip(1).collect();
                            tokens.push(Token::ClosingTag(tag_name));
                        } else {
                            tokens.push(Token::OpeningTag(String::from(tag_name), attr));
                        };

                        word = String::from("");
                    }

                    state = State::OutTag;
                }
                _ => word.push(c),
            };

            pos += 1;
        }

        if word.trim().chars().count() > 0 {
            tokens.push(Token::Text(String::from(word.trim())));
        }

        tokens.push(Token::End);

        tokens
    }
}

pub mod css {
    pub struct Stylesheet {
        // rules: Vec<Rules>,
    }

    pub struct Rule {
        selectors: Vec<Selector>,
        declarations: Vec<Declaration>,
    }

    enum Selector {
        Simple(SimpleSelector),
    }

    struct SimpleSelector {
        tag_name: Option<String>,
        id: Option<String>,
        class: Vec<String>,
    }

    struct Declaration {
        name: String,
        value: Value,
    }

    enum Value {
        Keyword(String),
        Length(f32, Unit),
        ColorValue(Color),
    }

    enum Unit {
        Px,
    }

    struct Color {
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    }

    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };

        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identifier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identifier());
                }
                '*' => {
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identifier());
                }
                _ => break,
            }
        }

        return selector;
    }

    pub type Specificity = (usize, usize, usize);

    impl Selector {
        pub fn specificity(&self) -> Specificity {
            let Selector::Simple(ref simple) = *self;
            let a = simple.id.iter().count();
            let b = simple.class.len();
            let c = simple.tag_name.iter().count();

            (a, b, c)
        }
    }
}

pub mod dom {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Node {
        children: Box<Vec<Node>>,
        node_type: NodeType,
    }

    pub fn print_tree(node: Node, level: usize) {
        let children = node.children.into_iter();
        for child in children {
            print_tree(child, level + 1)
        }

        println!("type: {:?}, level: {}", node.node_type, level);
    }

    #[derive(Debug, Clone, PartialEq)]
    enum NodeType {
        Text(String),
        Element(ElementData),
    }

    #[derive(Debug, Clone, PartialEq)]
    struct ElementData {
        tag_name: String,
        attributes: Vec<tokenizer::Attribute>,
    }

    pub fn text(data: String) -> Node {
        Node {
            children: Box::new(Vec::new()),
            node_type: NodeType::Text(data),
        }
    }

    use crate::tokenizer;
    pub fn elem(name: String, attrs: Vec<tokenizer::Attribute>, children: Vec<Node>) -> Node {
        Node {
            children: Box::new(children),
            node_type: NodeType::Element(ElementData {
                tag_name: name,
                attributes: attrs,
            }),
        }
    }

    pub fn create_parse_tree(html: String) -> Node {
        let tokens = tokenizer::tokenize(html);

        let mut stack: Vec<Node> = vec![];
        // let mut curr_node: Box<Option<Node>> = Box::new(None);
        for token in tokens {
            match token {
                tokenizer::Token::Start => {}
                tokenizer::Token::End => {}
                tokenizer::Token::OpeningTag(tag_name, attr) => {
                    stack.push(elem(tag_name, attr, vec![]));
                }
                tokenizer::Token::ClosingTag(tag_name) => {
                    let node = stack.pop().unwrap();
                    let mut parent = stack.pop();

                    if parent != None {
                        parent.as_mut().unwrap().children.push(node);
                    } else {
                        return node;
                    }

                    stack.push(parent.unwrap());
                }
                tokenizer::Token::Text(txt) => {
                    let node = text(txt);
                    let mut parent = stack.pop();

                    if parent != None {
                        parent.as_mut().unwrap().children.push(node);
                    } else {
                        return node;
                    }

                    stack.push(parent.unwrap());
                }
            }
        }

        unreachable!();
    }
}
