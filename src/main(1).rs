use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use chrono::NaiveDate;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};

#[derive(Parser, Debug)]
#[command(name = "bike_store_console", version, about)]
struct AppArgs {
    #[command(subcommand)]
    action: Actions,
}

#[derive(Subcommand, Debug)]
enum Actions {
    ShowStores,

    AddOrder {
        #[arg(long)]
        store: String,

        #[arg(long)]
        date: NaiveDate,

        #[arg(long)]
        quantity: i32,
    },

    OrdersStats {
        #[arg(long, default_value_t = 50)]
        limit: i64,
    },
}

async fn connect_db() -> Result<PgPool> {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL is missing. Create a .env file with DATABASE_URL.")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .context("Failed to connect to PostgreSQL. Check VPN, URL, and credentials.")?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = AppArgs::parse();
    let pool = connect_db().await?;

    match args.action {
        Actions::ShowStores => show_stores(&pool).await?,
        Actions::AddOrder { store, date, quantity } => add_order(&pool, store, date, quantity).await?,
        Actions::OrdersStats { limit } => orders_stats(&pool, limit).await?,
    }

    Ok(())
}


async fn show_stores(pool: &PgPool) -> Result<()> {
    let rows = sqlx::query(
        r#"
        SELECT name
        FROM bike_store.store
        ORDER BY name
        "#
    )
    .fetch_all(pool)
    .await
    .context("Failed to read stores table. Does bike_store.store exist?")?;

    if rows.is_empty() {
        println!("No stores found.");
        return Ok(());
    }

    println!("Stores:");
    for r in rows {
        let name: String = r.try_get("name")?;
        println!("- {}", name);
    }

    Ok(())
}

async fn add_order(pool: &PgPool, store: String, date: NaiveDate, quantity: i32) -> Result<()> {
    if quantity < 0 {
        bail!("Quantity must be >= 0");
    }

    let exists = sqlx::query(
        r#"
        SELECT 1
        FROM bike_store.store
        WHERE name = $1
        "#
    )
    .bind(&store)
    .fetch_optional(pool)
    .await
    .context("Failed to check if store exists.")?;

    if exists.is_none() {
        bail!("Store '{}' does not exist. Run `show-stores` to see valid names.", store);
    }


    let date_str = date.to_string(); 
    sqlx::query(
        r#"
        INSERT INTO bike_store.purchase_order (store_name, order_date, quantity)
        VALUES ($1, $2, $3)
        "#
    )
    .bind(&store)
    .bind(&date_str)
    .bind(quantity)
    .execute(pool)
    .await
    .context("Insert failed. Does bike_store.purchase_order exist?")?;

    println!("Ok: Added order: store='{}', date={}, quantity={}", store, date, quantity);
    Ok(())
}

async fn orders_stats(pool: &PgPool, limit: i64) -> Result<()> {
    let rows = sqlx::query(
        r#"
        SELECT
            store_name,
            COUNT(*)::bigint AS order_count,
            MAX(quantity) AS max_quantity,
            ROUND(AVG(quantity))::int AS avg_quantity
        FROM bike_store.purchase_order
        GROUP BY store_name
        ORDER BY order_count DESC, store_name
        LIMIT $1
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("Stats query failed.")?;

    if rows.is_empty() {
        println!("No orders found.");
        return Ok(());
    }

    println!("| {:15} | {:10} | {:12} | {:12} |", "Store", "#Orders", "Max", "Average");
    println!("|{:-<17}|{:-<12}|{:-<14}|{:-<14}|", "", "", "", "");

    for r in rows {
        let store_name: String = r.try_get("store_name")?;
        let order_count: i64 = r.try_get("order_count")?;
        let max_quantity: Option<i32> = r.try_get("max_quantity")?;
        let avg_quantity: Option<i32> = r.try_get("avg_quantity")?;

        println!(
            "| {:15} | {:10} | {:12} | {:12} |",
            store_name,
            order_count,
            max_quantity.unwrap_or(0),
            avg_quantity.unwrap_or(0)
        );
    }

    Ok(())
}
