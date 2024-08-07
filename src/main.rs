use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest};
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::fs;
use anyhow::{Context, Result};
use std::env;
use tokio::sync::Mutex;

async fn scrape_goodwill(client: &Client, query: &str, writer: &Mutex<BufWriter<File>>) -> Result<()> {
    let base_url = "https://www.goodwillfinds.com/search/?q=";
    let search_url = format!("{}{}", base_url, query);
    let mut start = 0;
    let sz = 48;
    let mut products_found;
    let max_pages = 10; // Limit to 10 pages
    let mut current_page = 0;

    while current_page < max_pages {
        let url = format!("{}&start={}&sz={}", search_url, start, sz);
        println!("Scraping URL: {}", url);

        let result = client.get(&url).send().await?;
        let body = result.text().await?;

        let document = Html::parse_document(&body);
        let product_selector = Selector::parse(".b-product_tile-actions").unwrap();

        products_found = 0;
        for product in document.select(&product_selector) {
            products_found += 1;
            if let Some(data_analytics) = product.value().attr("data-analytics") {
                if let Ok(analytics) = serde_json::from_str::<Value>(data_analytics) {
                    let name = analytics["name"].as_str().unwrap_or("N/A").to_string();
                    let price = analytics["price"].as_str().unwrap_or("N/A").to_string();
                    let description = analytics["category"].as_str().unwrap_or("N/A").to_string();

                    let mut writer = writer.lock().await;
                    writeln!(writer, "{},{},{}", name, price, description)?;
                }
            }
        }

        if products_found == 0 {
            break;
        }

        start += sz;
        current_page += 1;
    }

    Ok(())
}

async fn scrape_ebay(client: &Client, query: &str, writer: &Mutex<BufWriter<File>>) -> Result<()> {
    let base_url = "https://www.ebay.com/sch/i.html?_from=R40&_nkw=";
    let search_url = format!("{}{}", base_url, query);
    let mut page = 1;
    let mut products_found;
    let max_pages = 10; // Limit to 10 pages

    while page <= max_pages {
        let url = format!("{}&_sacat=0&_nls=2&_dmd=2&_ipg=240&_pgn={}", search_url, page);
        println!("Scraping URL: {}", url);

        let result = client.get(&url).send().await?;
        let body = result.text().await?;

        let document = Html::parse_document(&body);
        let product_selector = Selector::parse(".s-item").unwrap();
        let title_selector = Selector::parse(".s-item__title").unwrap();
        let price_selector = Selector::parse(".s-item__price").unwrap();
        let next_page_disabled_selector = Selector::parse(".pagination__next[aria-disabled='true']").unwrap();

        products_found = 0;
        for product in document.select(&product_selector) {
            products_found += 1;

            let raw_title = product.select(&title_selector).next()
                .map_or("N/A".to_string(), |n| n.text().collect::<Vec<_>>().join("").trim().to_string());

            let raw_price = product.select(&price_selector).next()
                .map_or("N/A".to_string(), |p| p.inner_html().trim().to_string());
            let price = raw_price.replace("<!--F#f_0-->", "")
                                 .replace("<!--F/-->", "")
                                 .trim()
                                 .to_string();

            let mut writer = writer.lock().await;
            writeln!(writer, "{},{}", raw_title, price)?;
        }

        let next_page_disabled = document.select(&next_page_disabled_selector).next();
        if products_found == 0 || next_page_disabled.is_some() {
            break;
        }

        page += 1;
    }

    Ok(())
}

async fn search(req: HttpRequest) -> impl Responder {
    let query_string = req.query_string();
    let query_value = query_string.split('=').nth(1).unwrap_or("").to_string();

    // Create a reqwest client
    let client = Client::new();

    // Define the file path
    let file_path = "output.csv";

    // Remove the old file if it exists
    if fs::metadata(file_path).is_ok() {
        fs::remove_file(file_path).unwrap();
    }

    // Create a new file and writer
    let file = File::create(file_path).unwrap();
    let writer = Mutex::new(BufWriter::new(file));

    // Perform the scraping
    let result = async {
        scrape_goodwill(&client, &query_value, &writer).await?;
        scrape_ebay(&client, &query_value, &writer).await;

        // Ensure all data is written to the file
        let mut writer = writer.lock().await;
        writer.flush().unwrap();

        Ok::<(), anyhow::Error>(())
    }.await;

    // Return a response based on the result
    match result {
        Ok(()) => HttpResponse::Ok().body(format!("Data scraped and saved to output.csv for query: {}", query_value)),
        Err(e) => HttpResponse::InternalServerError().body(format!("An error occurred: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Get the port from the environment variable or default to 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    HttpServer::new(|| {
        App::new()
            .route("/search", web::get().to(search))
    })
    .bind(format!("0.0.0.0:{}", port))? // Bind to all interfaces and use the port from the environment variable
    .run()
    .await
}
