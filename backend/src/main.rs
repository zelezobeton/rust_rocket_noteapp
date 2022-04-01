#[macro_use] extern crate rocket;

use sqlx::sqlite::SqlitePool;
use dotenv::dotenv;
use std::env;

use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::response::status;

mod structs;
use structs::{Note, NoteVector};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "GET, POST, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    let pool = SqlitePool::connect(&env::var("DATABASE_URL").expect("Failed getting db URL"))
        .await
        .expect("Failed to connect to database");

    rocket::build()
        .attach(CORS)
        .manage::<SqlitePool>(pool)
        .mount("/", routes![get_notes, sync_notes, handle_options]) 
}

// Routes //////////////////////////////////////////////////////////////////////////////////////////

#[get("/")]
async fn get_notes(pool: &rocket::State<SqlitePool>) -> String {
    let note_vec = list_notes(&pool).await.expect("Failed to list notes");
    match serde_json::to_string(&note_vec) {
        Ok(notes_str) => notes_str,
        Err(_) => "[]".to_string()
    }
}

#[post("/", data = "<notes>")]
async fn sync_notes(pool: &rocket::State<SqlitePool>, notes: String) -> status::Accepted<String> { 
    // Deal with individual notes from frontend by method  
    sort_notes(&pool, notes).await;

    // Respond with synced notes
    let note_vec = list_notes(&pool).await.expect("Failed to list notes");
    match serde_json::to_string(&note_vec) {
        Ok(notes_str) => status::Accepted(Some(notes_str)),
        Err(_) => status::Accepted(Some("[]".to_string()))
    }
}

// Important for handling CORS error
#[options("/")]
async fn handle_options() -> Status {
    Status::Ok
}

// CRUD functions //////////////////////////////////////////////////////////////////////////////////

async fn sort_notes(pool: &SqlitePool, notes: String) {
    // Convert the JSON string back to a NoteVector
    let note_vec: NoteVector = serde_json::from_str(&notes).unwrap();

    // Sort notes from frontend by method
    for note in note_vec {
        let method = note.method.as_str();

        match method {
            "" => continue,
            "CREATE" => {
                create_note(
                    &pool, note.created, note.changed, note.title, note.content, 
                ).await.expect("Failed writing note");
            }
            "UPDATE" => {
                update_note(&pool, note.id, note.changed, note.title, note.content).await.expect("Failed updating note");
            }
            "DELETE" => {
                delete_note(&pool, note.id).await.expect("Failed deleting note");
            }
            _ => {
                println!("Unknown method:{}", method);
            }
        }
    }
}

async fn list_notes(pool: &SqlitePool) -> anyhow::Result<NoteVector> {
    let recs = sqlx::query!(
        r#"
            SELECT id, created, changed, title, content
            FROM note_table
            ORDER BY changed DESC
        "#)
        .fetch_all(pool)
        .await
        .expect("Failed to list notes");

    let mut note_vec = NoteVector::new();
    for rec in recs {
        let note = Note {
            method: "".to_string(),
            id: rec.id, 
            created: rec.created, 
            changed: rec.changed, 
            title: rec.title, 
            content: rec.content 
        };
        note_vec.push(note);
    };

    Ok(note_vec)
}

async fn create_note(pool: &SqlitePool, created: i64, changed: i64, title: String, content: String) -> anyhow::Result<i64> {
    let mut conn = pool.acquire().await?;

    // Insert the task, then obtain the ID of this row
    let id = sqlx::query!(
        r#"
        INSERT INTO note_table ( created, changed, title, content )
        VALUES ( ?1, ?2, ?3, ?4 )
        "#,
        created, changed, title, content
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

async fn update_note(pool: &SqlitePool, id: i64, changed: i64, title: String, content: String) -> anyhow::Result<bool> {
    let rows_affected = sqlx::query!(
        r#"
        UPDATE note_table
        SET changed = ?1, title = ?2, content = ?3
        WHERE id = ?4
        "#,
        changed, title, content, id
    )
    .execute(pool)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
}

async fn delete_note(pool: &SqlitePool, id: i64) -> anyhow::Result<bool> {
    let rows_affected = sqlx::query!(
        r#"
        DELETE FROM note_table
        WHERE id = ?1
        "#,
        id
    )
    .execute(pool)
    .await?
    .rows_affected();

    Ok(rows_affected > 0)
}