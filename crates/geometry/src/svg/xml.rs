//! A bounded XML reader for SVG files: elements, attributes and nesting only.
//! Provenance: the W3C XML 1.0 syntax (tags, attributes, comments, CDATA, processing
//! instructions, DOCTYPE, character references); original implementation.
//!
//! Text content is not kept. Entity references other than the five predefined ones and
//! character references stay as written: a DOCTYPE entity is never expanded.

use super::{SvgError, invalid};

/// The deepest element nesting read; deeper input is refused, not followed.
pub(super) const MAX_DEPTH: usize = 256;

pub(super) struct Node {
    /// The local name (`svg:path` reads `path`).
    pub name: String,
    /// Qualified attribute names (`xlink:href`, `inkscape:label`) and decoded values.
    pub attrs: Vec<(String, String)>,
    pub children: Vec<usize>,
}

impl Node {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }
}

/// The element tree; node 0 is the root element.
pub(super) struct Tree {
    pub nodes: Vec<Node>,
}

struct Cursor<'a> {
    text: &'a str,
    at: usize,
}

impl Cursor<'_> {
    fn rest(&self) -> &str {
        self.text.get(self.at..).unwrap_or("")
    }

    fn starts(&self, prefix: &str) -> bool {
        self.rest().starts_with(prefix)
    }

    /// Move past the next `end`, or refuse naming `what`.
    fn skip_past(&mut self, end: &str, what: &str) -> Result<(), SvgError> {
        let found = self.rest().find(end).ok_or_else(|| invalid(format!("unterminated {what}")))?;
        self.at += found + end.len();
        Ok(())
    }

    fn skip_space(&mut self) {
        let rest = self.rest();
        self.at += rest.len() - rest.trim_start().len();
    }

    fn name(&mut self) -> &str {
        let rest = self.rest();
        let len = rest.find(|c: char| c.is_whitespace() || matches!(c, '/' | '>' | '=' | '<')).unwrap_or(rest.len());
        let start = self.at;
        self.at += len;
        self.text.get(start..self.at).unwrap_or("")
    }

    /// A DOCTYPE, with an internal subset in brackets if it has one.
    fn skip_doctype(&mut self) -> Result<(), SvgError> {
        let mut depth = 0usize;
        let mut quote: Option<char> = None;
        for (i, c) in self.rest().char_indices() {
            match (quote, c) {
                (Some(q), c) if c == q => quote = None,
                (Some(_), _) => {}
                (None, '"' | '\'') => quote = Some(c),
                (None, '[') => depth += 1,
                (None, ']') => depth = depth.saturating_sub(1),
                (None, '>') if depth == 0 => {
                    self.at += i + 1;
                    return Ok(());
                }
                _ => {}
            }
        }
        Err(invalid("unterminated DOCTYPE"))
    }
}

/// Decode the predefined entities and character references; leave any other reference as is.
fn decode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let Some(semi) = rest.find(';').filter(|&s| s <= 10) else {
            out.push('&');
            rest = &rest[1..];
            continue;
        };
        let body = &rest[1..semi];
        let ch = match body {
            "lt" => Some('<'),
            "gt" => Some('>'),
            "amp" => Some('&'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => body
                .strip_prefix("#x")
                .or_else(|| body.strip_prefix("#X"))
                .map_or_else(|| body.strip_prefix('#').and_then(|d| d.parse().ok()), |h| u32::from_str_radix(h, 16).ok())
                .and_then(char::from_u32),
        };
        match ch {
            Some(c) => {
                out.push(c);
                rest = &rest[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn local(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

/// Read the element tree of `text`, refusing more than `max_elements` elements.
pub(super) fn parse(text: &str, max_elements: usize) -> Result<Tree, SvgError> {
    let mut cur = Cursor { text, at: 0 };
    let mut nodes: Vec<Node> = Vec::new();
    // Open elements: node index and qualified name.
    let mut stack: Vec<(usize, String)> = Vec::new();
    let mut root_done = false;
    while let Some(lt) = cur.rest().find('<') {
        cur.at += lt;
        if cur.starts("<?") {
            cur.skip_past("?>", "processing instruction")?;
        } else if cur.starts("<!--") {
            cur.skip_past("-->", "comment")?;
        } else if cur.starts("<![CDATA[") {
            cur.skip_past("]]>", "CDATA section")?;
        } else if cur.starts("<!") {
            cur.skip_doctype()?;
        } else if cur.starts("</") {
            cur.at += 2;
            let name = cur.name().to_owned();
            cur.skip_space();
            if !cur.starts(">") {
                return Err(invalid(format!("unterminated closing tag </{name}")));
            }
            cur.at += 1;
            match stack.pop() {
                Some((_, open)) if open == name => root_done = stack.is_empty(),
                Some((_, open)) => return Err(invalid(format!("closing tag </{name}> does not match <{open}>"))),
                None => return Err(invalid(format!("closing tag </{name}> has no opening tag"))),
            }
        } else {
            cur.at += 1;
            let qualified = cur.name().to_owned();
            if qualified.is_empty() {
                return Err(invalid("a '<' that starts no tag"));
            }
            if root_done && stack.is_empty() {
                return Err(invalid("a second root element after the <svg> element"));
            }
            if nodes.len() >= max_elements {
                return Err(invalid(format!("more than {max_elements} elements: the file is larger than the import budget")));
            }
            if stack.len() >= MAX_DEPTH {
                return Err(invalid(format!("elements nested more than {MAX_DEPTH} deep")));
            }
            let attrs = attributes(&mut cur, &qualified)?;
            let index = nodes.len();
            nodes.push(Node { name: local(&qualified).to_owned(), attrs, children: Vec::new() });
            if let Some(p) = stack.last().and_then(|(parent, _)| nodes.get_mut(*parent)) {
                p.children.push(index);
            }
            if cur.starts("/>") {
                cur.at += 2;
                root_done |= stack.is_empty();
            } else {
                cur.at += 1; // '>'
                stack.push((index, qualified));
            }
        }
    }
    if let Some((_, open)) = stack.last() {
        return Err(invalid(format!("<{open}> is not closed")));
    }
    match nodes.first() {
        Some(root) if root.name == "svg" => Ok(Tree { nodes }),
        _ => Err(invalid("the file has no <svg> root element")),
    }
}

/// The attributes of a start tag; leaves the cursor on `>` or `/>`.
fn attributes(cur: &mut Cursor<'_>, tag: &str) -> Result<Vec<(String, String)>, SvgError> {
    let mut attrs: Vec<(String, String)> = Vec::new();
    loop {
        cur.skip_space();
        if cur.starts("/>") || cur.starts(">") {
            return Ok(attrs);
        }
        if cur.rest().is_empty() {
            return Err(invalid(format!("unterminated tag <{tag}")));
        }
        let name = cur.name().to_owned();
        if name.is_empty() {
            return Err(invalid(format!("unterminated tag <{tag}")));
        }
        cur.skip_space();
        if !cur.starts("=") {
            return Err(invalid(format!("attribute {name} of <{tag}> has no value")));
        }
        cur.at += 1;
        cur.skip_space();
        let quote = match cur.rest().chars().next() {
            Some(q @ ('"' | '\'')) => q,
            _ => return Err(invalid(format!("attribute {name} of <{tag}> must be quoted"))),
        };
        cur.at += 1;
        let len = cur.rest().find(quote).ok_or_else(|| invalid(format!("unterminated tag <{tag}")))?;
        let raw = cur.rest().get(..len).unwrap_or("");
        let value = decode(raw);
        cur.at += len + 1;
        if !attrs.iter().any(|(n, _)| *n == name) {
            attrs.push((name, value));
        }
    }
}
