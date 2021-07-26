use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use pulldown_cmark::html::push_html;
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag};
use serde::Serialize;
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
    let md = fs::read_to_string(path)?;

    let mut outline_tmp = Vec::new();
    let mut tmp = None;

    let options = Options::all();
    let parser = Parser::new_ext(&md, options).filter_map(|event| {
        match event {
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
            Event::Text(ref text) | Event::Code(ref text) => {
                if tmp.is_some() {
                    tmp.as_mut().unwrap().name += text;
                    // Some(event)
                    None
                } else {
                    Some(event)
                }
            }
            _ => Some(event),
        }
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
    build_dir(Path::new("./next"), Path::new("docs"))?;
    build_dir(Path::new("./versioned_docs"), Path::new("docs"))?;
    build_dir(Path::new("./posts"), Path::new("posts"))?;

    Ok(())
}
