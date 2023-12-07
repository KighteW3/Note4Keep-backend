use async_once_cell::Lazy;
use dotenv::dotenv;
use mongodb::{
    bson::Document,
    options::{ClientOptions, FindOptions},
    Client, Collection, Cursor,
};
use std::env;

pub struct AppState {
    pub db: mongodb::Client,
}

pub async fn connect_db() -> mongodb::Client {
    dotenv().ok();

    let uri = match env::var("MONGODB_URI") {
        Ok(key) => key,
        Err(_) => panic!("There is no mongodb uri"),
    };

    let mut client_options = if let Ok(parsed) = ClientOptions::parse(uri).await {
        parsed
    } else {
        panic!("No client options parsed")
    };

    client_options.app_name = Some("Note4Keep".to_string());

    match Client::with_options(client_options) {
        Ok(cli) => {
            println!("Database connected");
            cli
        }
        Err(e) => {
            panic!("{:?}", e)
        }
    }
}

pub async fn database_coll<T>(db: &mongodb::Client, coll: &str) -> Collection<T> {
    db.database("firstBackendProj").collection::<T>(coll)
}

/* pub async fn get_users<User>(
    db: &mongodb::Client,
    collection: &Collection<User>,
    query: Document,
    options: FindOptions,
) -> Result<Vec<User>, mongodb::error::Error> {
    let mut cursor = collection.await.find(query, options).await?;

    let mut result = Vec::new();

    while let Some(users) = cursor.try_next().await.unwrap() {
        result.push(users)
    }
    Ok(result)
} */
