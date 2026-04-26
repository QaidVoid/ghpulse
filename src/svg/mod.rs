pub mod theme;

/// SVG document builder with programmatic element creation.
#[allow(dead_code)]
pub struct Svg {
    width: u32,
    height: u32,
    elements: Vec<String>,
    styles: Vec<String>,
    defs: Vec<String>,
}

#[allow(dead_code)]
impl Svg {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            elements: Vec::new(),
            styles: Vec::new(),
            defs: Vec::new(),
        }
    }

    /// Add a CSS style block.
    pub fn style(&mut self, css: &str) {
        self.styles.push(css.to_string());
    }

    /// Add an SVG `<defs>` entry (filters, gradients, etc).
    pub fn def(&mut self, content: &str) {
        self.defs.push(content.to_string());
    }

    /// Create a `<rect>` element builder.
    pub fn rect(&self, x: f64, y: f64, w: f64, h: f64) -> ElementBuilder {
        ElementBuilder::new_self_closing(format!(
            r#"<rect x="{x}" y="{y}" width="{w}" height="{h}""#
        ))
    }

    /// Create a `<circle>` element builder.
    pub fn circle(&self, cx: f64, cy: f64, r: f64) -> ElementBuilder {
        ElementBuilder::new_self_closing(format!(r#"<circle cx="{cx}" cy="{cy}" r="{r}""#))
    }

    /// Create a `<line>` element builder.
    pub fn line(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> ElementBuilder {
        ElementBuilder::new_self_closing(format!(
            r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}""#
        ))
    }

    /// Create a `<text>` element builder with inline content.
    pub fn text(&self, x: f64, y: f64, content: &str) -> ElementBuilder {
        ElementBuilder::new_paired(
            format!(r#"<text x="{x}" y="{y}""#),
            format!("{content}</text>"),
        )
    }

    /// Create a `<path>` element builder.
    pub fn path(&self, d: &str) -> ElementBuilder {
        ElementBuilder::new_self_closing(format!(r#"<path d="{d}""#))
    }

    /// Add a `<g>` (group) element with inner content.
    pub fn group(&mut self, attrs: &str, content: &str) {
        self.elements.push(format!("<g {attrs}>{content}</g>"));
    }

    /// Add raw SVG content.
    pub fn raw(&mut self, svg: &str) {
        self.elements.push(svg.to_string());
    }

    /// Push a built element into the document.
    pub fn add(&mut self, el: ElementBuilder) {
        self.elements.push(el.build());
    }

    /// Render the complete SVG document as a string.
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        let mut out = String::with_capacity(4096);

        out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        out.push('\n');
        out.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            self.width, self.height, self.width, self.height
        ));
        out.push('\n');

        if !self.styles.is_empty() {
            out.push_str("<style>");
            for s in &self.styles {
                out.push_str(s);
            }
            out.push_str("</style>\n");
        }

        if !self.defs.is_empty() {
            out.push_str("<defs>");
            for d in &self.defs {
                out.push_str(d);
            }
            out.push_str("</defs>\n");
        }

        for el in &self.elements {
            out.push_str(el);
            out.push('\n');
        }

        out.push_str("</svg>");
        out
    }
}

/// Builder for adding attributes to an SVG element before closing it.
///
/// Handles both self-closing elements (like `<rect/>`) and paired elements
/// (like `<text>...</text>`).
#[allow(dead_code)]
pub struct ElementBuilder {
    /// Opening tag with attributes being built (e.g. `<rect x="0" ...`).
    opening: String,
    /// Closing portion: either empty (self-closing) or `>content</tag>`.
    closing: String,
}

#[allow(dead_code)]
impl ElementBuilder {
    /// Create a self-closing element (rect, circle, line, path).
    pub(crate) fn new_self_closing(tag: String) -> Self {
        Self {
            opening: tag,
            closing: String::new(),
        }
    }

    /// Create a paired element (text, etc).
    pub(crate) fn new_paired(tag: String, closing: String) -> Self {
        Self {
            opening: tag,
            closing,
        }
    }

    pub fn fill(mut self, color: &str) -> Self {
        self.opening.push_str(&format!(r#" fill="{color}""#));
        self
    }

    pub fn stroke(mut self, color: &str) -> Self {
        self.opening.push_str(&format!(r#" stroke="{color}""#));
        self
    }

    pub fn stroke_width(mut self, w: f64) -> Self {
        self.opening.push_str(&format!(r#" stroke-width="{w}""#));
        self
    }

    pub fn opacity(mut self, o: f64) -> Self {
        self.opening.push_str(&format!(r#" opacity="{o}""#));
        self
    }

    pub fn rx(mut self, r: f64) -> Self {
        self.opening.push_str(&format!(r#" rx="{r}""#));
        self
    }

    pub fn font_size(mut self, s: f64) -> Self {
        self.opening.push_str(&format!(r#" font-size="{s}""#));
        self
    }

    pub fn font_family(mut self, f: &str) -> Self {
        self.opening.push_str(&format!(r#" font-family="{f}""#));
        self
    }

    pub fn text_anchor(mut self, a: &str) -> Self {
        self.opening.push_str(&format!(r#" text-anchor="{a}""#));
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.opening.push_str(&format!(r#" class="{c}""#));
        self
    }

    pub fn filter(mut self, f: &str) -> Self {
        self.opening.push_str(&format!(r#" filter="url(#{f})""#));
        self
    }

    pub fn transform(mut self, t: &str) -> Self {
        self.opening.push_str(&format!(r#" transform="{t}""#));
        self
    }

    pub fn attr(mut self, key: &str, val: &str) -> Self {
        self.opening.push_str(&format!(r#" {key}="{val}""#));
        self
    }

    /// Finalize and return the SVG element string.
    pub fn build(self) -> String {
        if self.closing.is_empty() {
            format!("{}/>", self.opening)
        } else {
            format!("{}>{}", self.opening, self.closing)
        }
    }
}
