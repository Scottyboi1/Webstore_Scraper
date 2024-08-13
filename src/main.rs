use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest};
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::fs;
use anyhow::{Context, Result};
use std::env;

////////////////////FUNCTION TO SCRAPE GOODWILL////////////////////
async fn scrape_goodwill(client: &Client, query: &str) -> Result<String> {
    ////Initialize variables////
    let base_url = "https://www.goodwillfinds.com/search/?q="; //Goodwills base URL
    let search_url = format!("{}{}", base_url, query); //Search query given by user
    let mut start = 0; //Page start
    let sz = 48; //48 listings per page
    let mut products_found;
    let mut page_count = 0; //Keep track of pages scraped
    let mut output = String::new();

    ////Loop to scrape each page////
    loop {
        //Establish URL and print the URL being scraped//
        let url = format!("{}&start={}&sz={}", search_url, start, sz);
        println!("Scraping URL: {}", url);

        //Checks if URL is valid//
        let response = client.get(&url).send().await.context("Failed to fetch Goodwill URL")?;
        let body = response.text().await.context("Failed to get response text")?;

        //Parse HTML from the URL to get data//
        let document = Html::parse_document(&body);
        let product_selector = Selector::parse(".b-product_tile-actions").unwrap();

        products_found = 0;

        //Gets name, price and description of listings//
        for product in document.select(&product_selector) {
            products_found += 1;
            if let Some(data_analytics) = product.value().attr("data-analytics") {
                if let Ok(analytics) = serde_json::from_str::<Value>(data_analytics) {
                    let name = analytics["name"].as_str().unwrap_or("N/A").to_string();
                    let price = analytics["price"].as_str().unwrap_or("N/A").to_string();
                    let description = analytics["category"].as_str().unwrap_or("N/A").to_string();
                    
                    //Outputs listings in name, price, description format//
                    output.push_str(&format!("{},{},{}\n", name, price, description));
                }
            }
        }

        //Increment page count and start page #//
        page_count += 1;
        start += sz;

        //If no more listings are found, the program stops scraping goodwill//
        if products_found == 0 || page_count >= 10 {
            break;
        }
    }

    Ok(output)
}

////////////////////FUNCTION TO SCRAPE EBAY////////////////////
async fn scrape_ebay(client: &Client, query: &str) -> Result<String> {
    ////Initialize variables////
    let base_url = "https://www.ebay.com/sch/i.html?_from=R40&_nkw="; //Ebays base URL
    let search_url = format!("{}{}", base_url, query); //Search query given by user
    let mut page = 1; //Page start
    let mut products_found;
    let mut page_count = 0; //Keep track of pages scraped
    let mut output = String::new();

    ////Loop to scrape each page////
    loop {
        //Establish URL and print the URL being scraped//
        let url = format!("{}&_sacat=0&_nls=2&_dmd=2&_ipg=240&_pgn={}", search_url, page);
        println!("Scraping URL: {}", url);

        //Checks if URL is valid//
        let response = client.get(&url).send().await.context("Failed to fetch eBay URL")?;
        let body = response.text().await.context("Failed to get response text")?;

        //Parse HTML from the URL to get data//
        let document = Html::parse_document(&body);
        let product_selector = Selector::parse(".s-item").unwrap();
        let title_selector = Selector::parse(".s-item__title").unwrap();
        let price_selector = Selector::parse(".s-item__price").unwrap();
        let next_page_disabled_selector = Selector::parse(".pagination__next[aria-disabled='true']").unwrap();

        products_found = 0;
        //Gets name, price and description of listings//
        for product in document.select(&product_selector) {
            products_found += 1;
            //Deletes unnecessary strings in HTML//
            let raw_title = product.select(&title_selector).next()
                .map_or("N/A".to_string(), |n| n.text().collect::<Vec<_>>().join("").trim().to_string());
            let raw_price = product.select(&price_selector).next()
                .map_or("N/A".to_string(), |p| p.inner_html().trim().to_string());
            let price = raw_price.replace("<!--F#f_0-->", "")
                                 .replace("<!--F/-->", "")
                                 .trim()
                                 .to_string();
            //Outputs data in title, price format//
            output.push_str(&format!("{},{}\n", raw_title, price));
        }

        //Increment page cunt and page #//
        page_count += 1;
        page += 1;

        //If no more listings are found, the program stops scraping goodwill//
        let next_page_disabled = document.select(&next_page_disabled_selector).next();
        if products_found == 0 || next_page_disabled.is_some() || page_count >= 10 {
            break;
        }
    }

    Ok(output)
}

////Search function to handle HTTP requests and start scraping////
async fn search(req: HttpRequest) -> impl Responder {
    let query_string = req.query_string();
    let query_value = query_string.split('=').nth(1).unwrap_or("").to_string();

    //Create a reqwest client//
    let client = Client::new();

    //Start scraping Goodwill
    let goodwill_data = match scrape_goodwill(&client, &query_value).await {
        Ok(data) => data,
        _ => {
            println!("Goodwill scraping failed or timed out.");
            String::new()
        }
    };

    //Start scraping eBay
    let ebay_data = match scrape_ebay(&client, &query_value).await {
        Ok(data) => data,
        _ => {
            println!("eBay scraping failed or timed out.");
            String::new()
        }
    };

    //Combine the data from both scrapes//
    let combined_output = format!("{}\n{}", goodwill_data, ebay_data);

    //Write the output to a CSV file//
    let file_path = "output.csv";
    let mut file = File::create(file_path).unwrap();
    file.write_all(combined_output.as_bytes()).unwrap();

    //Read the output.csv file and return the data//
    match fs::read_to_string(file_path) {
        Ok(contents) => HttpResponse::Ok()
            .content_type("text/csv")
            .body(contents),
        Err(_) => HttpResponse::InternalServerError().body("Failed to read the output CSV file"),
    }
}

////Main function, handles HTTP server and route requests to search function////
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //Get the port from the environment variable or default to 8080//
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    HttpServer::new(|| {
        App::new()
            .route("/search", web::get().to(search))
    })
    .bind(format!("0.0.0.0:{}", port))? //Bind to all interfaces and use the port from the environment variable
    .run()
    .await
}
