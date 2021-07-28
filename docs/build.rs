use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, mem};

use pulldown_cmark::html::push_html;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use serde::Serialize;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use walkdir::WalkDir;

// Sync definition with website/src/content.rs
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct MarkdownPage {
    html: String,
    outline: Vec<Outline>,
}

// Sync definition with website/src/content.rs
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Outline {
    name: String,
    children: Vec<Outline>,
}

fn parse(path: &Path) -> Result<MarkdownPage, Box<dyn Error>> {
    // Syntect initialization.
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    builder.add_from_folder("./syntax", true)?;
    let ps = builder.build();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["InspiredGitHub"];

    let md = fs::read_to_string(path)?;

    let mut outline_tmp = Vec::new();
    let mut tmp = None;

    let mut code_block_buf = String::new();
    let mut inside_code_block = false;

    let options = Options::all();
    let parser = Parser::new_ext(&md, options).filter_map(|event| match event {
        Event::Start(Tag::Heading(level)) => {
            if level == 1 {
                Some(event)
            } else {
                tmp = Some(Outline {
                    name: String::new(),
                    children: Vec::new(),
                });
                None
            }
        }
        Event::End(Tag::Heading(level)) => {
            if level == 1 {
                Some(event)
            } else {
                let tmp = tmp.take().unwrap();
                let anchor = tmp.name.trim().to_lowercase().replace(" ", "-");
                let name = tmp.name.clone();
                if level == 2 {
                    outline_tmp.push(tmp);
                } else {
                    let l = outline_tmp
                        .last_mut()
                        .expect("cannot have non level 2 heading at root");
                    l.children.push(tmp);
                }
                Some(Event::Html(CowStr::from(format!(
                    "<h{} id=\"{}\">{}</h{}>",
                    level, anchor, name, level
                ))))
            }
        }
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_lang))) => {
            inside_code_block = true;

            None
        }
        Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
            inside_code_block = false;

            let code = mem::take(&mut code_block_buf);
            let syntax = ps
                .find_syntax_by_token(&lang)
                .unwrap_or_else(|| panic!("{} is an invalid language", lang));
            let highlighted_html = highlighted_html_for_string(&code, &ps, syntax, theme);

            Some(Event::Html(CowStr::from(highlighted_html)))
        }
        Event::Text(ref text) | Event::Code(ref text) => {
            if inside_code_block {
                code_block_buf.push_str(&text);
                None
            } else if tmp.is_some() {
                tmp.as_mut().unwrap().name += text;
                None
            } else {
                Some(event)
            }
        }
        _ => Some(event),
    });

    let mut html = String::new();
    push_html(&mut html, parser);

    Ok(MarkdownPage {
        html,
        outline: outline_tmp,
    })
}

fn build_dir(base: &Path, output: &Path) -> Result<(), Box<dyn Error>> {
    // Even though it is considered bad practice to write to a directory outside of OUT_DIR, we do
    // it anyways so that Trunk can copy it into the dist/ directory.
    let out_dir = Path::new("../website/static");

    for entry in WalkDir::new(base) {
        let entry = entry?;

        if entry.path().extension() == Some(&OsString::from_str("md").unwrap()) {
            // File is markdown.

            let page = parse(entry.path())?;
            let output_dir: PathBuf = out_dir.join(output);
            let output_path: PathBuf = output_dir
                .join(entry.path().strip_prefix(&base).unwrap())
                .with_extension("json");

            let output_json = serde_json::to_string(&page).unwrap();
            fs::create_dir_all(output_path.parent().unwrap())?;
            fs::write(output_path, output_json)?;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=next");
    println!("cargo:rerun-if-changed=versioned_docs");
    println!("cargo:rerun-if-changed=posts");

    build_dir(Path::new("./next"), Path::new("docs"))?;
    build_dir(Path::new("./versioned_docs"), Path::new("docs"))?;
    build_dir(Path::new("./posts"), Path::new("posts"))?;

    Ok(())
}
