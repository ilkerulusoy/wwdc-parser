use anyhow::{Context, Result};
use scraper::{Html, Selector, ElementRef};  // Added Element trait
use std::fs;
use std::fmt;
use clap::{Parser, ValueEnum};
use headless_chrome::{Browser, LaunchOptionsBuilder};

struct WWDCVideo {
    title: String,
    url: String,
    overview: String,
    transcript: String,
    code_samples: Vec<CodeSample>,
    resources: Vec<Resource>,
}

struct CodeSample {
    title: String,
    timestamp: String,
    code: String,
    language: String,
}

struct Resource {
    title: String,
    url: String,
    resource_type: ResourceType,
}

#[derive(Debug)]
enum ResourceType {
    Document,
    Download,
    Video,
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResourceType::Document => write!(f, "Documentation"),
            ResourceType::Download => write!(f, "Download"),
            ResourceType::Video => write!(f, "Video"),
        }
    }
}

impl WWDCVideo {
    fn to_markdown(&self) -> MarkdownOutput {
        let mut md = String::new();
        
        // Title and URL
        md.push_str(&format!("# {}\n", self.title));
        md.push_str(&format!("> {}\n\n", self.url));
        
        // Overview
        md.push_str("## Overview\n");
        md.push_str(&format!("{}\n\n", self.overview));
        
        // Resources
        if !self.resources.is_empty() {
            md.push_str("## Resources\n");
            for resource in &self.resources {
                md.push_str(&format!("- [{} ({})]({})\n", 
                    resource.title,
                    resource.resource_type,
                    resource.url
                ));
            }
            md.push_str("\n");
        }

        // Code Samples
        if !self.code_samples.is_empty() {
            md.push_str("## Code Samples\n");
            for sample in &self.code_samples {
                md.push_str(&format!("### {} ({})\n", sample.title, sample.timestamp));
                md.push_str(&format!("```{}\n{}\n```\n\n", sample.language, sample.code));
            }
        }

        // Transcript
        if !self.transcript.is_empty() {
            md.push_str("## Transcript\n");
            md.push_str(&format!("{}\n", self.transcript));
        }

        MarkdownOutput {
            content: md,
            title: self.title.clone(),
        }
    }
}

fn parse_wwdc_video(url: &str) -> Result<WWDCVideo> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-User", "?1")
        .send()?;

    let html = response.text()?;
    let document = Html::parse_document(&html);

    // Selectors
    let title_selector = Selector::parse("h1").unwrap();
    let overview_selector = Selector::parse(".supplement.details > p").unwrap();
    let transcript_selector = Selector::parse(".supplement.transcript .sentence").unwrap();
    let code_selector = Selector::parse(".sample-code-main-container").unwrap();
    let resources_selector = Selector::parse(".links.small li").unwrap();

    // Extract title and overview
    let title = document.select(&title_selector)
        .next()
        .context("Missing title")?
        .text()
        .collect::<String>();

    let overview = document.select(&overview_selector)
        .next()
        .context("Missing overview")?
        .text()
        .collect::<String>();

    // Extract transcript
    let transcript = document.select(&transcript_selector)
        .map(|element| element.text().collect::<String>())
        .collect::<Vec<String>>()
        .join(" ");

    // Extract code samples
    let code_samples = document.select(&code_selector)
        .map(|element| {
            let title_elem = element.select(&Selector::parse("p").unwrap()).next();
            let code_elem = element.select(&Selector::parse("code").unwrap()).next();
            
            if let (Some(title), Some(code)) = (title_elem, code_elem) {
                let title_text = title.text().collect::<String>();
                // Extract timestamp from title (format: "10:40 - Setting scene association behavior")
                let (timestamp, title) = if let Some(idx) = title_text.find(" - ") {
                    (&title_text[..idx], &title_text[idx + 3..])
                } else {
                    ("", &title_text[..])
                };

                Some(CodeSample {
                    title: title.to_string(),
                    timestamp: timestamp.to_string(),
                    code: code.text().collect::<String>(),
                    language: "swift".to_string(), // Default to Swift for WWDC
                })
            } else {
                None
            }
        })
        .flatten()
        .collect();

    // Extract resources
    let resources = document.select(&resources_selector)
        .map(|element: ElementRef| {
            let link = element.select(&Selector::parse("a").unwrap()).next()?;
        let url = link.value().attr("href")?;
        let title = link.text().collect::<String>();
        
        // Simplified class check
        let classes = element.value().attr("class").unwrap_or("");
        
        let resource_type = if classes.contains("document") {
            ResourceType::Document
        } else if classes.contains("download") {
            ResourceType::Download
        } else if classes.contains("video") {
            ResourceType::Video
        } else {
            ResourceType::Document // Default type
        };

        Some(Resource {
            title,
            url: url.to_string(),
            resource_type,
        })
    })
    .flatten()
    .collect();

    Ok(WWDCVideo {
        title,
        url: url.to_string(),
        overview,
        transcript,
        code_samples,
        resources,
    })
}

#[derive(Debug, Clone, ValueEnum)]
enum ContentType {
    Video,
    Document,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// URL of the WWDC content
    url: String,

    /// Type of content to parse
    #[arg(short, long, value_enum, default_value_t = ContentType::Video)]
    content_type: ContentType,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let markdown = match args.content_type {
        ContentType::Video => {
            let video = parse_wwdc_video(&args.url)?;
            video.to_markdown()
        }
        ContentType::Document => {
            let doc = parse_wwdc_document(&args.url)?;
            doc.to_markdown()
        }
    };
    
    let content_type = match args.content_type {
        ContentType::Video => "video",
        ContentType::Document => "doc",
    };
    
    let filename = format!("wwdc_{}_{}.md", 
        content_type,
        sanitize_filename(&markdown.title)
    );
    
    fs::write(&filename, markdown.content)?;
    println!("Generated markdown file: {}", filename);
    Ok(())
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|', ' '], "_")
        .to_lowercase()
        .trim()
        .to_string()
}

fn parse_wwdc_document(url: &str) -> Result<WWDCDocument> {
    // Start headless browser
    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .build()?;
    
    let browser = Browser::new(options)?;
    let tab = browser.new_tab()?;
    
    // Load page and wait for JavaScript execution
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;
    
    // Wait for JavaScript to load
    tab.wait_for_element("h1")?;
    
    // Get HTML content of the page
    let html = tab.get_content()?;
    let document = Html::parse_document(&html);

    // Get main title
    let title_selector = Selector::parse("h1").unwrap();
    let title = document.select(&title_selector)
        .next()
        .context("Title not found")?
        .text()
        .collect::<String>();

    // Get meta description
    let desc_selector = Selector::parse("meta[name='description']").unwrap();
    let description = document.select(&desc_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .unwrap_or_default()
        .to_string();

    // Get overview content
    let overview_selector = Selector::parse(".content > p").unwrap();
    let overview = document.select(&overview_selector)
        //.next()
        .map(|el| el.text().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n");
        //.unwrap_or_default();

    // Get notes
    let notes_selector = Selector::parse(".note").unwrap();
    let notes: Vec<String> = document.select(&notes_selector)
        .map(|note| {
            let label = note.select(&Selector::parse(".label").unwrap())
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_default();
            
            let content = note.select(&Selector::parse("p:not(.label)").unwrap())
                .map(|el| el.text().collect::<String>())
                .collect::<Vec<String>>()
                .join("\n");

            format!("{}: {}", label, content)
        })
        .collect();

    // Get sections
    let section_selector = Selector::parse(".contenttable-section").unwrap();
    let mut sections = Vec::new();

    for section in document.select(&section_selector) {
        let section_title = section
            .select(&Selector::parse(".contenttable-title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let items = section
            .select(&Selector::parse(".link-block").unwrap())
            .map(|item| {

                // Get title from either identifier or decorated-title
                let title = item
                    .select(&Selector::parse(".identifier, .decorated-title, code").unwrap())
                    .next()
                    .map(|el| {
                        // Remove <wbr> tags and collect text
                        el.text().collect::<Vec<_>>().join("")
                            .replace("\u{200B}", "") // Remove zero-width space if any
                    })
                    .unwrap_or_else(|| {
                        // Fallback to span text if no identifier/decorated-title
                        item.select(&Selector::parse(".link span").unwrap())
                            .next()
                            .map(|el| el.text().collect::<Vec<_>>().join(""))
                            .unwrap_or_default()
                    });

                DocumentItem {
                    title,
                    description: item.select(&Selector::parse(".content").unwrap())
                        .next()
                        .map(|el| el.text().collect())
                        .unwrap_or_default(),
                    url: item.select(&Selector::parse("a").unwrap())
                        .next()
                        .and_then(|el| el.value().attr("href"))
                        .map(|href| format!("https://developer.apple.com{}", href))
                        .unwrap_or_default(),
                    item_type: item.select(&Selector::parse(".decorator").unwrap())
                        .next()
                        .map(|el| el.text().collect())
                        .unwrap_or_else(|| "article".to_string()),
                }
            })
            .collect();

        sections.push(Section {
            title: section_title,
            items,
        });
    }

    Ok(WWDCDocument {
        title,
        description,
        overview,
        notes,
        sections,
    })
}

// Add WWDCDocument struct
struct WWDCDocument {
    title: String,
    description: String,
    overview: String,
    notes: Vec<String>,
    sections: Vec<Section>,
}

struct Section {
    title: String,
    items: Vec<DocumentItem>,
}

struct DocumentItem {
    title: String,
    description: String,
    url: String,
    item_type: String,
}

// Önce yeni struct'ı ekleyelim
struct MarkdownOutput {
    content: String,
    title: String,
}

impl WWDCDocument {
    fn to_markdown(&self) -> MarkdownOutput {
        let mut md = String::new();
        
        // Title and description
        md.push_str(&format!("# {}\n\n", self.title));
        md.push_str(&format!("{}\n\n", self.description));
        
        // Overview
        md.push_str("## Overview\n");
        md.push_str(&format!("{}\n\n", self.overview));
        
        // Notes
        if !self.notes.is_empty() {
            md.push_str("## Notes\n");
            for note in &self.notes {
                md.push_str(&format!("{}\n\n", note));
            }
        }
        
        // Sections
        for section in &self.sections {
            md.push_str(&format!("## {}\n\n", section.title));
            
            for item in &section.items {
                md.push_str(&format!("### {} `{}`\n", item.item_type, item.title));
                md.push_str(&format!("{}\n\n", item.description));
                md.push_str(&format!("[Documentation]({})\n\n", item.url));
            }
        }
        
        MarkdownOutput {
            content: md,
            title: self.title.clone(),
        }
    }
}
