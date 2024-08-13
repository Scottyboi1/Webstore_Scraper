use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest};
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::fs::File;
use std::io::{Write};
use std::fs;
use anyhow::{Context, Result};
use std::env;

async fn scrape_goodwill(client: &Client, query: &str) -> Result<String> {
    let base_url = "https://www.goodwillfinds.com/search/?q=";
    let search_url = format!("{}{}", base_url, query);
    let sz = 48; // Number of items per page
    let mut output = String::new();

    for page in 0..10 { // Scrape the first 10 pages
        let start = page * sz;
        let url = format!("{}&start={}&sz={}", search_url, start, sz);
        println!("Scraping URL: {}", url);

        let response = client.get(&url).send().await.context("Failed to fetch Goodwill URL")?;
        let body = response.text().await.context("Failed to get response text")?;

        let document = Html::parse_document(&body);
        let product_selector = Selector::parse(".b-product_tile-actions").unwrap();

        for product in document.select(&product_selector) {
            if let Some(data_analytics) = product.value().attr("data-analytics") {
                if let Ok(analytics) = serde_json::from_str::<Value>(data_analytics) {
                    let name = analytics["name"].as_str().unwrap_or("N/A").to_string();
                    let price = analytics["price"].as_str().unwrap_or("N/A").to_string();
                    let description = analytics["category"].as_str().unwrap_or("N/A").to_string();

                    output.push_str(&format!("{},{},{}\n", name, price, description));
                }
            }
        }
    }

    Ok(output)
}

async fn scrape_ebay(client: &Client, query: &str) -> Result<String> {
    let base_url = "https://www.ebay.com/sch/i.html?_from=R40&_nkw=";
    let search_url = format!("{}{}", base_url, query);
    let mut output = String::new();

    for page in 1..=10 { // Scrape the first 10 pages
        let url = format!("{}&_sacat=0&_nls=2&_dmd=2&_ipg=240&_pgn={}", search_url, page);
        println!("Scraping URL: {}", url);

        let response = client.get(&url).send().await.context("Failed to fetch eBay URL")?;
        let body = response.text().await.context("Failed to get response text")?;

        let document = Html::parse_document(&body);
        let product_selector = Selector::parse(".s-item").unwrap();
        let title_selector = Selector::parse(".s-item__title").unwrap();
        let price_selector = Selector::parse(".s-item__price").unwrap();

        for product in document.select(&product_selector) {
            let raw_title = product.select(&title_selector).next()
                .map_or("N/A".to_string(), |n| n.text().collect::<Vec<_>>().join("").trim().to_string());

            let raw_price = product.select(&price_selector).next()
                .map_or("N/A".to_string(), |p| p.inner_html().trim().to_string());
            let price = raw_price.replace("<!--F#f_0-->", "")
                                 .replace("<!--F/-->", "")
                                 .trim()
                                 .to_string();

            output.push_str(&format!("{},{}\n", raw_title, price));
        }
    }

    Ok(output)
}

async fn search(req: HttpRequest) -> impl Responder {
    let query_string = req.query_string();
    let query_value = query_string.split('=').nth(1).unwrap_or("").to_string();

    // Create a reqwest client
    let client = Client::new();

    // Scrape Goodwill (first 10 pages)
    let goodwill_data = match scrape_goodwill(&client, &query_value).await {
        Ok(data) => data,
        Err(e) => {
            println!("Goodwill scraping failed: {}", e);
            String::new()  // Proceed even if Goodwill fails
        }
    };

    // Scrape eBay (first 10 pages)
    let ebay_data = match scrape_ebay(&client, &query_value).await {
        Ok(data) => data,
        Err(e) => {
            println!("eBay scraping failed: {}", e);
            String::new()  // Proceed even if eBay fails
        }
    };

    // Combine the data from both scrapes
    let combined_output = format!("{}\n{}", goodwill_data, ebay_data);

    // Write the combined output to the CSV file
    let file_path = "output.csv";
    let mut file = File::create(file_path).unwrap();
    file.write_all(combined_output.as_bytes()).unwrap();

    // Read the output.csv file and return its contents
    match fs::read_to_string(file_path) {
        Ok(contents) => HttpResponse::Ok()
            .content_type("text/csv")
            .body(contents),
        Err(_) => HttpResponse::InternalServerError().body("Failed to read the output CSV file"),
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
