//! A small strict element tree over quick-xml's pull parser, for the game's XML data files (DLC lists, setup and
//! content files). A UTF-8 byte order mark is skipped; the five predefined entities and character references are
//! resolved; comments and the declaration are skipped; anything else unusual (a DOCTYPE, processing instructions,
//! unknown entities, text beside the root) is an error.
#![deny(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation
)]
use quick_xml::events::Event;
use quick_xml::{Reader, XmlVersion};

use crate::{Result, invalid};

/// One element: where it starts, its name, attributes, child elements in order, and its text (concatenated, untrimmed).
#[derive(Clone, Debug, Default)]
pub struct Element {
    /// The byte offset of the element's start tag in the document it was read from (for [`crate::rbf`]'s elements, of
    /// its first record): where errors about it point.
    pub offset: u64,
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Element>,
    pub text: String,
}

impl Element {
    /// An attribute by name.
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// The first child of this name (element names compare without case, as the game's parser hashes them).
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|c| c.name.eq_ignore_ascii_case(name))
    }

    /// The trimmed text of the child of this name.
    pub fn child_text(&self, name: &str) -> Option<&str> {
        self.child(name).map(|c| c.text.trim())
    }

    /// The `value` attribute of the child of this name.
    pub fn child_value(&self, name: &str) -> Option<&str> {
        self.child(name).and_then(|c| c.attr("value"))
    }

    /// The `Item` children (any case).
    pub fn items(&self) -> impl Iterator<Item = &Element> {
        self.children
            .iter()
            .filter(|c| c.name.eq_ignore_ascii_case("item"))
    }

    /// Fails naming the first child whose name is not in `known` (compared without case).
    pub fn only(&self, what: &str, known: &[&str]) -> Result<()> {
        match self
            .children
            .iter()
            .find(|c| !known.iter().any(|k| c.name.eq_ignore_ascii_case(k)))
        {
            Some(c) => Err(invalid(
                "XML",
                c.offset,
                format!(
                    "{what}: unknown element <{}> in <{}>, expected {}{}",
                    c.name,
                    self.name,
                    if known.len() == 1 { "" } else { "one of " },
                    known
                        .iter()
                        .map(|k| format!("<{k}>"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )),
            None => Ok(()),
        }
    }
}

/// The deepest element nesting read. The tree is built iteratively but dropped and walked by recursion, which a document
/// of a few hundred thousand nested elements overflows the stack of; the game's files nest a dozen deep.
pub const MAX_DEPTH: usize = 64;

fn err(what: &str, offset: usize, e: impl std::fmt::Display) -> crate::Error {
    invalid(
        "XML",
        u64::try_from(offset).unwrap_or(u64::MAX),
        format!("{what}: {e}"),
    )
}

/// Parses `bytes` (UTF-8, optionally with a byte order mark) into its root element. Errors name the byte offset in
/// `bytes` of the markup that is wrong.
pub fn parse(bytes: &[u8], what: &str) -> Result<Element> {
    let (bom, bytes) = match bytes.strip_prefix(b"\xEF\xBB\xBF") {
        Some(rest) => (3usize, rest),
        None => (0, bytes),
    };
    let text = std::str::from_utf8(bytes).map_err(|e| {
        err(
            what,
            bom.saturating_add(e.valid_up_to()),
            format!("not UTF-8 (expected UTF-8 text): {e}"),
        )
    })?;
    let mut reader = Reader::from_str(text);
    // end tags are matched here, so that a mismatch names both tags and the offset of the end tag
    reader.config_mut().check_end_names = false;
    let mut stack: Vec<Element> = Vec::new();
    let mut root: Option<Element> = None;
    loop {
        // the previous event ended where this one starts: the `<` of a tag, the `&` of a reference
        let at = bom.saturating_add(usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX));
        let ev = reader.read_event().map_err(|e| {
            let failed = bom.saturating_add(usize::try_from(reader.error_position()).unwrap_or(usize::MAX));
            err(what, failed.max(at), e)
        })?;
        let empty = matches!(ev, Event::Empty(_));
        match ev {
            Event::Decl(_) | Event::Comment(_) => {}
            Event::Start(s) | Event::Empty(s) => {
                if root.is_some() {
                    return Err(err(
                        what,
                        at,
                        format!(
                            "a second root element <{}>, expected the end of the document",
                            s.name().as_ref()
                        ),
                    ));
                }
                let name = s.name().as_ref().to_string();
                let mut attrs = Vec::new();
                for a in s.attributes() {
                    let a = a.map_err(|e| err(what, at, format!("<{name}>: {e}")))?;
                    let k = a.key.as_ref().to_string();
                    let v = a
                        .normalized_value(XmlVersion::Implicit1_0)
                        .map_err(|e| err(what, at, format!("<{name} {k}=...>: {e}")))?
                        .into_owned();
                    attrs.push((k, v));
                }
                let el = Element {
                    offset: u64::try_from(at).unwrap_or(u64::MAX),
                    name,
                    attrs,
                    children: Vec::new(),
                    text: String::new(),
                };
                if empty {
                    match stack.last_mut() {
                        Some(p) => p.children.push(el),
                        None => root = Some(el),
                    }
                } else {
                    if stack.len() >= MAX_DEPTH {
                        return Err(err(
                            what,
                            at,
                            format!(
                                "elements nest deeper than {MAX_DEPTH} (<{}>), the most read",
                                el.name
                            ),
                        ));
                    }
                    stack.push(el);
                }
            }
            Event::End(e) => {
                let found = e.name().as_ref().to_string();
                let el = stack.pop().ok_or_else(|| {
                    err(
                        what,
                        at,
                        format!("</{found}> with no element open, expected the end of the document"),
                    )
                })?;
                if found != el.name {
                    return Err(err(
                        what,
                        at,
                        format!("</{found}> closes <{}>, expected </{}>", el.name, el.name),
                    ));
                }
                match stack.last_mut() {
                    Some(p) => p.children.push(el),
                    None => root = Some(el),
                }
            }
            Event::Text(t) => {
                let s = t.xml10_content();
                match stack.last_mut() {
                    Some(p) => p.text.push_str(&s),
                    None if s.trim().is_empty() => {}
                    None => {
                        return Err(err(
                            what,
                            at,
                            "text outside the root element, expected only whitespace",
                        ));
                    }
                }
            }
            Event::CData(c) => {
                let s = c.as_ref().to_string();
                stack
                    .last_mut()
                    .ok_or_else(|| err(what, at, "CDATA outside the root element"))?
                    .text
                    .push_str(&s);
            }
            Event::GeneralRef(r) => {
                let c = match r.resolve_char_ref().map_err(|e| err(what, at, e))? {
                    Some(c) => c,
                    None => match &*r {
                        "lt" => '<',
                        "gt" => '>',
                        "amp" => '&',
                        "apos" => '\'',
                        "quot" => '"',
                        other => {
                            return Err(err(
                                what,
                                at,
                                format!(
                                    "unknown entity &{other};, expected &lt; &gt; &amp; &apos; &quot; or a character reference"
                                ),
                            ));
                        }
                    },
                };
                stack
                    .last_mut()
                    .ok_or_else(|| err(what, at, "an entity outside the root element"))?
                    .text
                    .push(c);
            }
            Event::PI(_) | Event::DocType(_) => {
                return Err(err(
                    what,
                    at,
                    "processing instruction or DOCTYPE, which the game's files do not use",
                ));
            }
            Event::Eof => break,
        }
    }
    if let Some(open) = stack.last() {
        return Err(err(
            what,
            usize::try_from(open.offset).unwrap_or(usize::MAX),
            format!(
                "<{}> is not closed, expected </{}> before the end of the document",
                open.name, open.name
            ),
        ));
    }
    root.ok_or_else(|| err(what, bom.saturating_add(bytes.len()), "no root element"))
}

#[cfg(test)]
#[allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    reason = "fixtures, not untrusted bytes"
)]
mod tests {
    use super::*;

    /// Nesting is capped, and a document of hundreds of thousands of nested elements is refused, not a stack overflow.
    #[test]
    fn nesting_is_capped() {
        let nested = |depth: usize| format!("{}{}", "<a>".repeat(depth), "</a>".repeat(depth));
        assert!(parse(nested(MAX_DEPTH).as_bytes(), "t").is_ok());
        let e = parse(nested(MAX_DEPTH + 1).as_bytes(), "t")
            .unwrap_err()
            .to_string();
        assert!(e.contains("nest deeper than 64"), "{e}");
        assert!(parse(nested(400_000).as_bytes(), "t").is_err());
    }

    #[test]
    fn tree_entities_bom_and_strictness() {
        let doc = b"\xEF\xBB\xBF<?xml version=\"1.0\"?>\n<!-- c --><Root a=\"x&amp;y\"><Item>p&lt;q&#x41;</Item><item v=\"1\"/></Root>";
        let r = parse(doc, "t").expect("parses");
        assert_eq!(r.attr("a"), Some("x&y"));
        let items: Vec<_> = r.items().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text, "p<qA");
        assert_eq!(items[1].attr("v"), Some("1"));
        assert!(r.only("t", &["item"]).is_ok());
        assert!(r.only("t", &["other"]).is_err());
        assert!(parse(b"<a>&nope;</a>", "t").is_err());
        assert!(parse(b"<a></b>", "t").is_err());
        assert!(parse(b"<a/><b/>", "t").is_err());
        assert!(parse(b"<!DOCTYPE a><a/>", "t").is_err());
        assert!(parse(b"<a>", "t").is_err());
    }
}
