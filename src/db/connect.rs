use dotenv::dotenv;
use mongodb::{options::ClientOptions, Client, Collection};
use std::env;

pub const USERS: &str = "users";
pub const NOTES: &str = "notes";

pub struct DbState {
    pub db: mongodb::Client,
}

pub async fn connect_db() -> mongodb::Client {
    dotenv().ok();

    let uri = match env::var("MONGODB_URI") {
        Ok(key) => key,
        Err(_) => panic!("Error: There is no mongodb uri"),
    };

    let mut client_options = if let Ok(parsed) = ClientOptions::parse(uri).await {
        parsed
    } else {
        panic!("Error: No client options parsed")
    };

    client_options.app_name = Some("Note4Keep".to_string());

    match Client::with_options(client_options) {
        Ok(cli) => {
            println!("Database connected");
            cli
        }
        Err(e) => {
            panic!("Error: {:?}", e)
        }
    }
}

pub async fn database_coll<T>(db: &mongodb::Client, coll: &str) -> Collection<T> {
    db.database("firstBackendProj").collection::<T>(coll)
}

/* pub async fn get_users<User>(
    db: &mongodb::Client,
    query: Document,
    options: FindOptions,
) -> Result<Vec<User>, mongodb::error::Error> {
    let user_coll = database_coll::<User>(&db, "users").await;

    let find_options = FindOptions::builder().sort(doc! {}).build();

    let mut cursor = if let Ok(cursor) = user_coll.find(None, find_options).await {
        cursor
    } else {
        panic!("Error")
    };

    let mut result = Vec::new();

    while let Some(users) = cursor.try_next().await.unwrap() {
        result.push(users)
    }

    Ok(result)
} */
