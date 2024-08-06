use actix_web::{web, App, HttpServer, Responder, HttpRequest};
use reqwest::blocking::get;
use scraper::{Html, Selector};
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::fs;
use anyhow::{Context, Result};

fn scrape_goodwill(query: &str, writer: &mut dyn Write) -> Result<()> {
    let base_url = "https://www.goodwillfinds.com/search/?q=";
    let search_url = format!("{}{}", base_url, query);
    let mut start = 0;
    let sz = 48;
    let mut products_found;

    loop {
        let url = format!("{}&start={}&sz={}", search_url, start, sz);
        println!("Scraping URL: {}", url);

        let response = get(&url).context("Failed to fetch Goodwill URL")?;
        let body = response.text().context("Failed to get response text")?;

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

                    writeln!(writer, "{},{},{}", name, price, description)?;
                }
            }
        }

        if products_found == 0 {
            break;
        }

        start += sz;
    }

    Ok(())
}

fn scrape_ebay(query: &str, writer: &mut dyn Write) -> Result<()> {
    let base_url = "https://www.ebay.com/sch/i.html?_from=R40&_nkw=";
    let search_url = format!("{}{}", base_url, query);
    let mut page = 1;
    let mut products_found;

    loop {
        let url = format!("{}&_sacat=0&_nls=2&_dmd=2&_ipg=240&_pgn={}", search_url, page);
        println!("Scraping URL: {}", url);

        let response = get(&url).context("Failed to fetch eBay URL")?;
        let body = response.text().context("Failed to get response text")?;

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
    let query_string = req.query_string().to_string();
    let query_value = query_string.split('=').nth(1).unwrap_or("").to_string();

    // Clone the query_value for use inside the blocking closure
    let query_value_clone = query_value.clone();

    let result = web::block(move || {
        let file_path = "output.csv";
        if fs::metadata(file_path).is_ok() {
            fs::remove_file(file_path).unwrap();
        }

        let file = File::create(file_path).unwrap();
        let mut writer = BufWriter::new(file);

        writeln!(writer, "Name,Price,Description").unwrap();

        // Use the cloned query_value for scraping
        scrape_goodwill(&query_value_clone, &mut writer)?;
        scrape_ebay(&query_value_clone, &mut writer)?;

        writer.flush().unwrap();

        Ok::<_, anyhow::Error>(())
    }).await;

    // Return a response based on the result
    match result {
        Ok(Ok(())) => format!("Data scraped and saved to output.csv for query: {}", query_value),
        Ok(Err(e)) => format!("An error occurred: {}", e),
        Err(e) => format!("Failed to execute: {:?}", e),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/search", web::get().to(search))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
