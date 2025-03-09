use reqwest;
use scraper::{Html, Selector};
use sqlx::{sqlite::{SqlitePool, SqlitePoolOptions}};
use std::error::Error;
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct Country {
    name: String,
    capital: String,
    population: Option<i32>,
    area: Option<f64>,
}

async fn scrape_countries() -> Result<Vec<Country>, Box<dyn Error>> {
    let res = reqwest::get("https://www.scrapethissite.com/pages/simple/")
        .await?
        .text()
        .await?;

    let doc = Html::parse_document(&res);
    let country_selector = Selector::parse(".country")?;
    let name_selector = Selector::parse(".country-name")?;
    let capital_selector
        = Selector::parse(".country-capital")?;
    let population_selector
        = Selector::parse(".country-population")?;
    let area_selector
        = Selector::parse(".country-area")?;

    let mut countries = Vec::new();

    for element in doc.select(&country_selector) {
        let name = element
            .select(&name_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let capital = element
            .select(&capital_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let population = element
            .select(&population_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().parse::<i32>().ok())
            .flatten();

        let area = element
            .select(&area_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().parse::<f64>().ok())
            .flatten();

        countries.push( Country {
            name: name,
            capital: capital,
            population: population,
            area: area,
        });

        sleep(Duration::from_millis(500)).await
    }

    Ok(countries)
}

async fn store_countries(pool: &SqlitePool, countries: Vec<Country>) -> Result<(), Box<dyn Error>> {
    for country in countries {
        if country.name.is_empty() || country.capital.is_empty() {
            continue;
        }

        sqlx::query!(
            r#"
            INSERT INTO countries (name, capital, population, area)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(name) DO UPDATE SET
                capital = excluded.capital,
                population = excluded.population,
                area = excluded.area
            "#,
            country.name,
            country.capital,
            country.population,
            country.area
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
            .iter()
            .map(|c| c.name.clone())
            .collect();
        let expected_countries = vec![
            "Canada".to_string(),
            "United States".to_string(),
            "Mexico".to_string(),
        ];
        let andorra = countries
            .into_iter()
            .find(|c| c.name == "Andorra")
            .expect("Andorra was not found in the scraped countries");


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

        // Assert Andorra's details
        assert_eq!(
            andorra.capital,
            "Andorra la Vella",
            "Expected the capital of Andorra to be 'Andorra la Vella', but found '{}'",
            andorra.capital
        );
        assert_eq!(
            andorra.population,
            Some(84000),
            "Expected the population of Andorra to be '84000', but found '{:?}'",
            andorra.population
        );
        assert_eq!(
            andorra.area,
            Some(468.0),
            "Expected the area of Andorra to be '468.0', but found '{:?}'",
            andorra.area
        );
    }
}