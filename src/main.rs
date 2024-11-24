use anyhow::{Context, Result};
use scraper::{Html, Selector, ElementRef};  // Element trait'ini ekledik
use std::fs;
use std::fmt;

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
    fn to_markdown(&self) -> String {
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

        md
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
        
        // Basitleştirilmiş class kontrolü
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

fn main() -> Result<()> {
    // Get URL from command line arguments
    let url = std::env::args()
        .nth(1)
        .context("Usage: wwdc-parser <video-url>")?;
    
    let video = parse_wwdc_video(&url)?;
    
    let markdown = video.to_markdown();
    let filename = format!("wwdc_{}.md", sanitize_filename(&video.title));
    fs::write(&filename, markdown)?;
    
    println!("Generated markdown file: {}", filename);
    Ok(())
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-")
        .to_lowercase()
        .trim()
        .to_string()
}