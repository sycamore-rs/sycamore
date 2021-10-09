use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::{fs, mem};

use pulldown_cmark::html::push_html;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use serde::Serialize;
use syntect::highlighting::ThemeSet;
use syntect::html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use walkdir::WalkDir;

static HOSTNAME: &str = "https://sycamore-rs.netlify.app";

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
                    "<h{level} id=\"{anchor}\">{name}</h{level}>",
                    level = level,
                    anchor = anchor,
                    name = name
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

            let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
                syntax,
                &ps,
                syntect::html::ClassStyle::SpacedPrefixed { prefix: "s-" },
            );
            for line in LinesWithEndings::from(&code) {
                html_generator.parse_html_for_line_which_includes_newline(line);
            }
            let highlighted_html = html_generator.finalize();

            Some(Event::Html(CowStr::from(format!(
                "<pre>{}</pre>",
                highlighted_html
            ))))
        }
        Event::Text(ref text) | Event::Code(ref text) => {
            if inside_code_block {
                code_block_buf.push_str(text);
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

    for entry in WalkDir::new(base).sort_by_file_name() {
        let entry = entry?;

        if entry.path().extension() == Some(OsStr::new("md")) {
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

fn generate_sitemap_for_dir(
    buf: &mut impl Write,
    base: &str,
    dir: &Path,
    changefreq: &str,
    priority: &str,
) -> Result<(), Box<dyn Error>> {
    for entry in WalkDir::new(dir)
        .into_iter()
        .map(|e| e.unwrap())
        .filter(|e| e.path().is_file())
    {
        let path = entry.path().strip_prefix(&dir).unwrap().with_extension("");
        let path_str = path.iter().fold(String::new(), |acc, c| {
            format!("{}/{}", acc, c.to_str().unwrap())
        });

        write_url(buf, &format!("{}{}", base, path_str), changefreq, priority)?;
    }

    Ok(())
}

fn write_url(
    buf: &mut impl Write,
    path: &str,
    changefreq: &str,
    priority: &str,
) -> Result<(), Box<dyn Error>> {
    writeln!(
        buf,
        "<url>\
                <loc>{hostname}{path}</loc>\
                <changefreq>{changefreq}</changefreq>\
                <priority>{priority}</priority>\
            </url>",
        hostname = HOSTNAME,
        path = path,
        changefreq = changefreq,
        priority = priority,
    )?;
    Ok(())
}

fn generate_sitemap_xml() -> Result<(), Box<dyn Error>> {
    let out_path = Path::new("../website/sitemap_index.xml");

    let mut buf = String::new();
    writeln!(buf, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        buf,
        r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#
    )?;

    write_url(&mut buf, "", "monthly", "1.0")?;
    write_url(&mut buf, "/news", "monthly", "0.8")?;
    write_url(&mut buf, "/versions", "monthly", "0.3")?;

    // News
    generate_sitemap_for_dir(&mut buf, "/news", Path::new("posts"), "yearly", "0.8")?;

    // Docs for master
    generate_sitemap_for_dir(&mut buf, "/docs", Path::new("./next"), "weekly", "0.5")?;

    // Versioned docs
    for dir in fs::read_dir(Path::new("./versioned_docs"))? {
        let dir = dir?;
        let path = dir.path();
        if path.is_dir() {
            let version = path.file_name().unwrap().to_str().unwrap();
            generate_sitemap_for_dir(
                &mut buf,
                &format!("/docs/{}", version),
                &dir.path(),
                "yearly",
                "0.3",
            )?;
        }
    }

    // Examples
    for dir in fs::read_dir(Path::new("../examples"))? {
        let dir = dir?;
        let path = dir.path();
        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap();
            write_url(&mut buf, &format!("/examples/{}", name), "monthly", "0.5")?;
        }
    }

    writeln!(buf, r#"</urlset>"#)?;

    fs::write(out_path, buf)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=next");
    println!("cargo:rerun-if-changed=versioned_docs");
    println!("cargo:rerun-if-changed=posts");

    // Sitemap.
    generate_sitemap_xml()?;

    // Markdown files.
    build_dir(Path::new("./next"), Path::new("docs"))?;
    build_dir(Path::new("./versioned_docs"), Path::new("docs"))?;
    build_dir(Path::new("./posts"), Path::new("posts"))?;

    // Docs sidebars.
    let next_sidebar = fs::read_to_string("./next/sidebar.json")?;
    fs::write("../website/static/docs/sidebar.json", next_sidebar)?;
    for entry in WalkDir::new("./versioned_docs") {
        let entry = entry?;
        if entry.path().file_name() == Some(OsStr::new("sidebar.json")) {
            let sidebar = fs::read_to_string(entry.path())?;
            fs::write(
                Path::new("../website/static/docs/")
                    .join(entry.path().strip_prefix("./versioned_docs")?),
                sidebar,
            )?;
        }
    }

    // Syntax highlighting CSS files.
    let ts = ThemeSet::load_defaults();
    let dark_theme = &ts.themes["base16-ocean.dark"];
    let light_theme = &ts.themes["InspiredGitHub"];

    let dark_css =
        css_for_theme_with_class_style(dark_theme, ClassStyle::SpacedPrefixed { prefix: "s-" });
    let light_css =
        css_for_theme_with_class_style(light_theme, ClassStyle::SpacedPrefixed { prefix: "s-" });
    fs::write("../website/static/dark.css", dark_css)?;
    fs::write("../website/static/light.css", light_css)?;

    Ok(())
}
