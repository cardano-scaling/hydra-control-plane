use rocket::get;

#[get("/health")]
pub async fn health() -> String {
    "OK".to_string()
}
