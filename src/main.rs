use reqwest;
use scraper::{Html, Selector};
use sqlx::{sqlite::{SqlitePool, SqlitePoolOptions}};
use std::error::Error;

#[derive(Debug)]
struct Country {
    name: String,
}

async fn scrape_countries() -> Result<Vec<Country>, Box<dyn Error>> {
    let res = reqwest::get("https://www.scrapethissite.com/pages/simple/")
        .await?
        .text()
        .await?;

    let doc = Html::parse_document(&res);
    let country_selector = Selector::parse(".country-name")?;

    let countries = doc
        .select(&country_selector)
        .map(|element| Country {
            name: element.text().collect::<String>().trim().to_string(),
        })
        .collect();

    Ok(countries)
}

async fn store_countries(pool: &SqlitePool, countries: Vec<Country>) -> Result<(), Box<dyn Error>> {
    for country in countries {
        sqlx::query!(
            r#"
            INSERT INTO countries (name) VALUES (?1)
            "#,
            country.name
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up the database connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:scraper.db")
        .await?;

    // Scrape countries from the website
    println!("Scraping countries...");
    let countries = scrape_countries().await?;
    println!("Scraped countries: {:?}", countries);

    // Store countries in the database
    println!("Storing countries in the database...");
    store_countries(&pool, countries).await?;
    println!("Countries stored successfully!");

    Ok(())
}
