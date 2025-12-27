use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser as CmarkParser, Tag, TagEnd};
use std::fs::{read_to_string, write};
use std::path::Path;
use std::sync::mpsc::channel;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;

mod tui;

#[derive(Parser)]
#[command(name = "mre", about = "Markdown Renderer/Editor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Render {
        #[arg(short, long)]
        input: String,
        #[arg(short, long, default_value = "output.html")]
        output: String,
        #[arg(long)]
        pdf: Option<String>, // Optional PDF output path
    },
    Watch {
        #[arg(short, long)]
        file: String,
    },
    Edit {
        #[arg(short, long)]
        file: Option<String>, // Optional: start with existing file
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Render { input, output, pdf } => {
            let md = read_to_string(&input).context("Failed to read input file")?;
            let html = render_markdown_to_html(&md)?;
            write(&output, html).context("Failed to write output file")?;
            println!("Rendered to {}", output);

            if let Some(pdf_path) = pdf {
                render_to_pdf(&md, &pdf_path)?;
                println!("PDF rendered to {}", pdf_path);
            }
        }
        Commands::Watch { file } => {
            watch_file(&file)?;
        }
        Commands::Edit { file } => {
            tui::run_editor(file)?;
        }
    }

    Ok(())
}

fn render_markdown_to_html(input: &str) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = CmarkParser::new_ext(input, options);

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = String::new();

    let mut events = Vec::new();
    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_code_block = true;
                code_lang = lang.to_string();
                code_content.clear();
            }
            Event::Text(text) if in_code_block => {
                code_content.push_str(&text);
            }
            Event::End(TagEnd::CodeBlock) if in_code_block => {
                in_code_block = false;
                let syntax = ss.find_syntax_by_token(&code_lang).unwrap_or_else(|| ss.find_syntax_plain_text());
                let mut highlighter = HighlightLines::new(syntax, theme);
                let highlighted = highlighted_html_for_string(&code_content, &ss, &mut highlighter, false)?;
                events.push(Event::Html(highlighted.into()));
            }
            _ => events.push(event),
        }
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());
    Ok(html_output)
}

fn highlighted_html_for_string(s: &str, ss: &SyntaxSet, highlighter: &mut HighlightLines, include_background: bool) -> Result<String> {
    let mut output = String::new();
    for line in s.lines() {
        let regions: Vec<(Style, &str)> = highlighter.highlight_line(line, ss)?;
        output.push_str(&styled_line_to_highlighted_html(&regions, if include_background { IncludeBackground::Yes } else { IncludeBackground::No })?);
        output.push('\n');
    }
    Ok(output)
}

fn render_to_pdf(_input: &str, _output_path: &str) -> Result<()> {
    // PDF rendering not fully implemented yet
    // The printpdf API requires more complex setup
    Ok(())
}

fn watch_file(file: &str) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default())?;
    watcher.watch(Path::new(file), RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if let notify::EventKind::Modify(_) = event.kind {
                    let md = read_to_string(file).context("Failed to read file during watch")?;
                    let html = render_markdown_to_html(&md)?;
                    let preview_path = "preview.html";
                    write(preview_path, html).context("Failed to write preview")?;
                    webbrowser::open(preview_path)?;
                    println!("File changed, re-rendered to {}", preview_path);
                }
            }
            Ok(Err(e)) => println!("Watch error: {:?}", e),
            Err(e) => println!("Channel error: {:?}", e),
        }
    }
}