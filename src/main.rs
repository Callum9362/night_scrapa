use reqwest;
use scraper::{Html, Selector};
use sqlx::{sqlite::{SqlitePool, SqlitePoolOptions}};
use std::error::Error;
use std::time::Instant;

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
            INSERT OR IGNORE INTO countries (name) VALUES (?1)
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
    let start = Instant::now();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:scraper.db")
        .await?;

    println!("Scraping countries...");
    let countries = scrape_countries().await?;

    println!("Storing countries in the database...");
    store_countries(&pool, countries).await?;
    println!("Countries stored successfully!");

    let duration = start.elapsed();
    println!("Total time to scrape and store countries: {:.2?}", duration);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scrape_countries() {
        let countries = scrape_countries()
            .await
            .unwrap();
        let country_names: Vec<String> = countries
            .into_iter()
            .map(|c| c.name)
            .collect();
        let expected_countries = vec![
            "Canada".to_string(),
            "United States".to_string(),
            "Mexico".to_string(),
        ];

        for expected in expected_countries {
            assert!(
                country_names.contains(&expected),
                "Expected country '{}' not found in the scraped results",
                expected
            );
        }

        assert_eq!(
            country_names.len(),
            250,
            "Expected 250 countries, but found {}",
            country_names.len()
        );
    }
}