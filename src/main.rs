use std::fs::File;
use std::io::Write;

use regex::Regex;
use futures_util::StreamExt;
use scraper::{Html, Selector};
use indicatif::{ProgressBar, ProgressStyle};

async fn request_episode(ep: String, ep_name: String) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://dbz.watch-dbs.com/v/".to_string();
    let url = url + &ep + ".mp4";

    let mut response_stream = 
        reqwest::get(url)
        .await?
        .bytes_stream()
    ;

    let filename = "./downloads/".to_string();
    let filename = filename + &ep_name + ".mp4";
    let mut file = File::create(&filename).expect("Failure to create file!");

    while let Some(chunk) = response_stream.next().await {        
        match chunk {
            Ok(chunk) => {
                let write_file = 
                    file
                    .write_all(&chunk)
                    .or(Err(format!("Error while writing to file")))
                ;

                match write_file {
                    Err(e) => println!("{:?}", e),
                    _ => ()
                }
            }
            Err(e) => {
                println!("Error: {}", e)
            }
        }
    }

    Ok(())
}

async fn download_episode(ep: usize, ep_name: &str) -> std::io::Result<()> {
    let mut episode = ep.to_string();
    let non_alphanumeric = Regex::new(r"[^a-zA-Z0-9 ]").unwrap();
    let parsed_ep_name = non_alphanumeric.replace_all(ep_name, "");
    
    if ep < 10 {
        let ep_incremented = ep + 1;
        episode = "0".to_string() + &ep_incremented.to_string();
    }

    match request_episode(episode.to_string(), parsed_ep_name.to_string()).await {
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
        request_stack.push(download_episode(i, &episode_names[i]).await?);
    }

    Ok(())
}