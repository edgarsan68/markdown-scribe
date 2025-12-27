use pulldown_cmark::{html, Parser, Options};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, StyleModifier};
use syntect::html::highlighted_html_for_string;

fn render_markdown_to_html(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    let parser = Parser::new_ext(input, options);

    // Basic syntax highlighting setup
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    // For code blocks, you'd integrate highlighting here (expand later)
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn main() {
    let md = "# Hello\n```rust\nfn main() {}\n```";
    let html = render_markdown_to_html(md);
    println!("{}", html);
}


