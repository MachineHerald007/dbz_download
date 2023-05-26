use std::fs;
use std::fs::File;
use std::io::Write;

use regex::Regex;
use console::style;
use futures_util::StreamExt;
use scraper::{Html, Selector};
use reqwest::header::HeaderValue;
use indicatif::{ProgressBar, ProgressStyle};

pub trait HeaderValueExt {
    fn to_string(&self) -> String;
}

impl HeaderValueExt for HeaderValue {
    fn to_string(&self) -> String {
        self.to_str().unwrap_or_default().to_string()
    }
}

async fn request_episode(ep: String, ep_name: String, total_ep: String) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://dbz.watch-dbs.com/v/";
    let url = url.to_string() + &ep + ".mp4";
    let response = 
        reqwest::get(url)
        .await?
    ;

    match response.status() {
        reqwest::StatusCode::BAD_REQUEST => println!(
            "content-length:{:?} server:{:?}", 
            response.headers().get(reqwest::header::CONTENT_LENGTH),
            response.headers().get(reqwest::header::SERVER)
        ),
        _status => {
            let content_length = response.headers().get(reqwest::header::CONTENT_LENGTH).unwrap();
            let content_length = content_length.to_string().parse::<u64>()?; 
            let done_msg = style("DONE").bold().green();
            let bar_msg =
                style("\n  Download").bold().magenta().to_string() +
                &style(" [".to_owned() + &ep + "/" + &total_ep + "]").bold().yellow().to_string();
            let bar =
                ProgressBar::new(content_length);
                ProgressBar::println(&bar, bar_msg.to_string());
                ProgressBar::set_style(
                    &bar,
                    ProgressStyle::with_template(
                        "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}"
                    ).unwrap()
                );

            let stream = &mut response.bytes_stream();
            let _dir = fs::create_dir_all("./downloads")?;
            let filename = "./downloads/";
            let filename = filename.to_string() + &ep_name + ".mp4";
            let mut file = File::create(&filename).expect("Failure to create file!");

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(chunk) => {
                        let write_file =
                            file
                            .write_all(&chunk)
                            .or(Err(format!("Error while writing to file")))
                        ;

                        match write_file {
                            Err(e) => println!("{:?}", e),
                            _ => {
                                bar.inc(chunk.len().try_into().unwrap());
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }

            bar.finish_with_message(done_msg.to_string());
        }
    }

    Ok(())
}

async fn download_episode(ep: usize, ep_name: &str, total_ep: usize) -> std::io::Result<()> {
    let ep_incremented = ep + 1;
    let mut episode = ep_incremented.to_string();
    let non_alphanumeric = Regex::new(r"[^a-zA-Z0-9 ]").unwrap();
    let parsed_ep_name = non_alphanumeric.replace_all(ep_name, "").to_string();
    
    if ep_incremented < 10 {
        episode = "0".to_string() + &episode.to_string();
    } else {
        episode = ep_incremented.to_string()
    }

    match request_episode(episode, parsed_ep_name, total_ep.to_string()).await {
        Ok(()) => (),
        Err(e) => println!("{:?}", e)
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request_stack = Vec::new();
    let mut episode_names = Vec::new();

    let response =
        reqwest::get("https://watch-dbz52.funonline.co.in/dragon-ball-z/")
        .await?
        .text()
        .await?
    ;

    let document = Html::parse_document(&response);
    let ul_selector = Selector::parse("ul").unwrap();
    let li_selector = Selector::parse("li").unwrap();
    let episode_list_selector = Selector::parse(".episode-list").unwrap();
    let episode_list = document.select(&episode_list_selector).next().unwrap();
    let ul = episode_list.select(&ul_selector).next().unwrap();

    for node in ul.select(&li_selector) {
        episode_names.push(node.text().collect::<Vec<_>>().join(" "));
    }

    for i in 0..episode_names.len() {
        request_stack.push(download_episode(i, &episode_names[i], episode_names.len()).await?);
    }

    Ok(())
}