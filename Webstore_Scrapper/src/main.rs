use reqwest::blocking::get;
use scraper::{Html, Selector};
use std::error::Error;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::fs;
use serde_json::Value;

fn scrape_goodwill(base_url: &str, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    let mut start = 0;
    let sz = 48;
    let mut products_found;

    loop {
        let url = format!("{}?start={}&sz={}", base_url, start, sz);
        println!("Scraping URL: {}", url);

        // Fetch the webpage
        let response = get(&url)?;
        let body = response.text()?;

        // Parse the HTML
        let document = Html::parse_document(&body);
        let product_selector = Selector::parse(".b-product_tile-actions").unwrap();

        // Scrape product data
        products_found = 0;
        for product in document.select(&product_selector) {
            products_found += 1;
            // Extract the data-analytics attribute
            if let Some(data_analytics) = product.value().attr("data-analytics") {
                // Parse the JSON data
                if let Ok(analytics) = serde_json::from_str::<Value>(data_analytics) {
                    let name = analytics["name"].as_str().unwrap_or("N/A").to_string();
                    let price = analytics["price"].as_str().unwrap_or("N/A").to_string();
                    let description = analytics["category"].as_str().unwrap_or("N/A").to_string();

                    // Write to file
                    writeln!(writer, "{},{},{}", name, price, description)?;
                }
            }
        }

        // Break if no more products are found on the current page
        if products_found == 0 {
            break;
        }

        // Increment the start for the next page
        start += sz;
    }

    Ok(())
}

fn scrape_ebay(base_url: &str, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    let mut page = 1;
    let mut products_found;

    loop {
        let url = format!("{}&_pgn={}", base_url, page);
        println!("Scraping URL: {}", url);

        // Fetch the webpage
        let response = get(&url)?;
        let body = response.text()?;

        // Parse the HTML
        let document = Html::parse_document(&body);
        let product_selector = Selector::parse(".s-item").unwrap();
        let title_selector = Selector::parse(".s-item__title").unwrap();
        let price_selector = Selector::parse(".s-item__price").unwrap();
        let next_page_selector = Selector::parse(".pagination__next").unwrap();
        let next_page_disabled_selector = Selector::parse(".pagination__next[aria-disabled='true']").unwrap();

        // Scrape product data
        products_found = 0;
        for product in document.select(&product_selector) {
            products_found += 1;

            // Extract and clean title
            let raw_title = product.select(&title_selector).next()
                .map_or("N/A".to_string(), |n| n.text().collect::<Vec<_>>().join("").trim().to_string());

            // Extract and clean price
            let raw_price = product.select(&price_selector).next()
                .map_or("N/A".to_string(), |p| p.inner_html().trim().to_string());
            let price = raw_price.replace("<!--F#f_0-->", "")
                                 .replace("<!--F/-->", "")
                                 .trim()
                                 .to_string();

            // Write to file
            writeln!(writer, "{},{}", raw_title, price)?;
        }

        // Check for "next page" button and if it's disabled
        let next_page_disabled = document.select(&next_page_disabled_selector).next();

        // Break if no more products are found on the current page or the next page button is disabled
        if products_found == 0 || next_page_disabled.is_some() {
            break;
        }

        // Increment the page for the next iteration
        page += 1;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // List of URLs to scrape
    let goodwill_url = "https://www.goodwillfinds.com/electronics/computers-and-tablets/desktop/";
    let ebay_url = "https://www.ebay.com/sch/i.html?_from=R40&_nkw=desktop&_sacat=0&LH_ItemCondition=3000&LH_BIN=1&_udhi=60&_oaa=1&Form%2520Factor=Mini%2520Pc%7CMini%2520Desktop&_dcat=179&rt=nc&_ipg=240";

    // Delete the old CSV file if it exists
    let file_path = "output.csv";
    if fs::metadata(file_path).is_ok() {
        fs::remove_file(file_path)?;
    }

    // Create a new file and writer
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    // Write headers
    writeln!(writer, "Name,Price,Description")?;

    // Scrape data from each URL
    scrape_goodwill(goodwill_url, &mut writer)?;
    scrape_ebay(ebay_url, &mut writer)?;

    // Flush the writer to ensure all data is written
    writer.flush()?;

    println!("Data scraped and saved to output.csv");
    Ok(())
}
