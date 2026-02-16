use clap::Parser;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    id: String,
    #[arg(short, long)]
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite://skrin.db").await?;

    let args = Args::parse();

    let token = Uuid::new_v4().to_string();
    let user_id = args.id;
    let name = args.name;
    let created_at = chrono::Utc::now().to_rfc3339();
    let permissions = r#"["upload", "view"]"#;

    sqlx::query!(
        "INSERT INTO tokens (token, user_id, name, created_at, permissions)
        VALUES (?, ?, ?, ?, ?)",
        token,
        user_id,
        name,
        created_at,
        permissions
    )
    .execute(&pool)
    .await?;

    println!("Token for {}: {}", name, token);

    Ok(())
}
