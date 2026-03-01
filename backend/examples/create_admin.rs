/// One-time utility to create an admin user.
/// Usage: cargo run --example create_admin -- <username> <password>
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: cargo run --example create_admin -- <username> <password>");
        std::process::exit(1);
    }

    let username = &args[1];
    let password = &args[2];

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    // Connect to DB
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Insert user
    let row = sqlx::query!(
        r#"
        INSERT INTO admin_users (username, password_hash, email)
        VALUES ($1, $2, $3)
        ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash
        RETURNING id, username
        "#,
        username,
        hash,
        format!("{}@blocweather.local", username),
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert admin user");

    println!("✓ Admin user created: {} ({})", row.username, row.id);
}
