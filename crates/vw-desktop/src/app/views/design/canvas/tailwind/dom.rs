use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TailwindNode {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<TailwindNode>,
    pub text: Option<String>,
}

impl TailwindNode {
    pub fn new(tag: &str) -> Self {
        Self { tag: tag.to_string(), attributes: HashMap::new(), children: Vec::new(), text: None }
    }

    pub fn text(content: &str) -> Self {
        Self {
            tag: "text".to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
            text: Some(content.to_string()),
        }
    }

    pub fn to_html(&self, indent: usize) -> String {
        if let Some(text) = &self.text {
            return format!("{}{}", " ".repeat(indent), text);
        }

        let spaces = " ".repeat(indent);
        let opening_tag = format!("{}{}", spaces, self.opening_tag());

        if self.children.is_empty() {
            if is_void_tag(&self.tag) {
                return format!("{} />", opening_tag);
            }
            return format!("{}></{}>", opening_tag, self.tag);
        }

        if self.children.iter().any(|child| child.text.is_some()) {
            let content =
                self.children.iter().map(TailwindNode::to_inline_html).collect::<Vec<_>>().join("");
            return format!("{}>{}</{}>", opening_tag, content, self.tag);
        }

        let mut html = format!("{}>\n", opening_tag);
        for child in &self.children {
            html.push_str(&child.to_html(indent + 2));
            html.push('\n');
        }
        html.push_str(&format!("{}</{}>", spaces, self.tag));
        html
    }

    fn to_inline_html(&self) -> String {
        if let Some(text) = &self.text {
            return text.clone();
        }

        let mut html = self.opening_tag();
        if self.children.is_empty() {
            if is_void_tag(&self.tag) {
                html.push_str(" />");
            } else {
                html.push_str(&format!("></{}>", self.tag));
            }
            return html;
        }

        html.push('>');
        for child in &self.children {
            html.push_str(&child.to_inline_html());
        }
        html.push_str(&format!("</{}>", self.tag));
        html
    }

    fn opening_tag(&self) -> String {
        let mut html = format!("<{}", self.tag);

        let mut attrs: Vec<_> = self.attributes.iter().collect();
        attrs.sort_by_key(|a| a.0);

        for (key, value) in attrs {
            html.push_str(&format!(" {}=\"{}\"", key, value));
        }

        html
    }
}

fn is_void_tag(tag: &str) -> bool {
    matches!(tag, "img" | "br" | "hr" | "input")
}

pub fn nodes_to_html(nodes: &[TailwindNode]) -> String {
    nodes.iter().map(|n| n.to_html(0)).collect::<Vec<_>>().join("\n")
}

pub fn get_node_by_path<'a>(nodes: &'a [TailwindNode], path: &[usize]) -> Option<&'a TailwindNode> {
    if path.is_empty() {
        return None;
    }
    let root_idx = path[0];
    if let Some(root) = nodes.get(root_idx) {
        let mut current = root;
        for &idx in &path[1..] {
            if let Some(child) = current.children.get(idx) {
                current = child;
            } else {
                return None;
            }
        }
        return Some(current);
    }
    None
}

pub fn remove_node_by_path(nodes: &mut Vec<TailwindNode>, path: &[usize]) -> bool {
    if path.is_empty() {
        return false;
    }

    if path.len() == 1 {
        let idx = path[0];
        if idx < nodes.len() {
            nodes.remove(idx);
            return true;
        }
        return false;
    }

    let root_idx = path[0];
    let Some(mut current) = nodes.get_mut(root_idx) else {
        return false;
    };

    for &idx in &path[1..path.len() - 1] {
        let Some(child) = current.children.get_mut(idx) else {
            return false;
        };
        current = child;
    }

    let last = *path.last().unwrap_or(&0);
    if last < current.children.len() {
        current.children.remove(last);
        true
    } else {
        false
    }
}

pub fn parse_html(html: &str) -> Vec<TailwindNode> {
    let mut roots = Vec::new();
    let mut stack: Vec<TailwindNode> = Vec::new();

    let mut chars = html.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '<' {
            chars.next(); // consume '<'

            // Comment
            if chars.clone().take(3).collect::<String>() == "!--" {
                // Skip comment
                while let Some(c) = chars.next() {
                    if c == '-' && chars.clone().take(2).collect::<String>() == "->" {
                        chars.next();
                        chars.next();
                        break;
                    }
                }
                continue;
            }

            // Closing tag
            if let Some(&'/') = chars.peek() {
                chars.next(); // consume '/'
                let mut tag_name = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '>' {
                        chars.next();
                        break;
                    }
                    tag_name.push(chars.next().unwrap());
                }

                // Pop from stack
                if let Some(node) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    } else {
                        roots.push(node);
                    }
                }
                continue;
            }

            // Opening tag
            let mut tag_name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '>' || c == '/' {
                    break;
                }
                tag_name.push(chars.next().unwrap());
            }

            let mut node = TailwindNode::new(&tag_name);

            // Attributes
            loop {
                // Skip whitespace
                while let Some(&c) = chars.peek() {
                    if !c.is_whitespace() {
                        break;
                    }
                    chars.next();
                }

                if let Some(&c) = chars.peek() {
                    if c == '>' || c == '/' {
                        break;
                    }

                    // Parse attribute name
                    let mut attr_name = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '=' || c.is_whitespace() || c == '>' || c == '/' {
                            break;
                        }
                        attr_name.push(chars.next().unwrap());
                    }

                    // Parse attribute value
                    let mut attr_value = String::new();
                    if let Some(&'=') = chars.peek() {
                        chars.next(); // consume '='
                        if let Some(&'"') = chars.peek() {
                            chars.next(); // consume '"'
                            while let Some(&c) = chars.peek() {
                                if c == '"' {
                                    chars.next();
                                    break;
                                }
                                attr_value.push(chars.next().unwrap());
                            }
                        } else if let Some(&'\'') = chars.peek() {
                            chars.next(); // consume '\''
                            while let Some(&c) = chars.peek() {
                                if c == '\'' {
                                    chars.next();
                                    break;
                                }
                                attr_value.push(chars.next().unwrap());
                            }
                        } else {
                            // Unquoted value
                            while let Some(&c) = chars.peek() {
                                if c.is_whitespace() || c == '>' || c == '/' {
                                    break;
                                }
                                attr_value.push(chars.next().unwrap());
                            }
                        }
                    }

                    if !attr_name.is_empty() {
                        node.attributes.insert(attr_name, attr_value);
                    }
                } else {
                    break;
                }
            }

            // Check self-closing
            let mut is_self_closing = false;
            if let Some(&'/') = chars.peek() {
                chars.next();
                is_self_closing = true;
            }
            if let Some(&'>') = chars.peek() {
                chars.next();
            }

            if is_self_closing
                || tag_name == "img"
                || tag_name == "br"
                || tag_name == "hr"
                || tag_name == "input"
            {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    roots.push(node);
                }
            } else {
                stack.push(node);
            }
        } else {
            // Text content
            let mut text = String::new();
            while let Some(&c) = chars.peek() {
                if c == '<' {
                    break;
                }
                text.push(chars.next().unwrap());
            }

            if !text.trim().is_empty() {
                let text_node = TailwindNode::text(&text);
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(text_node);
                } else {
                    // Top level text? usually ignored or wrapped
                    roots.push(text_node);
                }
            }
        }
    }

    // Clean up remaining stack
    while let Some(node) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(node);
        } else {
            roots.push(node);
        }
    }

    roots
}

#[cfg(test)]
#[path = "dom_tests.rs"]
mod tests;
