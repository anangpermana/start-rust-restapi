#[macro_use] extern crate rocket;
use rocket::{fairing::AdHoc, serde::json::Json, State};
use serde::{Deserialize, Serialize};

#[macro_use] extern crate diesel;
use diesel::{prelude::*, table, Insertable, Queryable};
use rocket_sync_db_pools::database;

table! {
    blog_posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}

#[database("my_db")]
pub struct Db(diesel::PgConnection);

#[derive(Serialize, Deserialize, Queryable, Debug, Insertable)]
#[table_name = "blog_posts"]
struct BlogPost {
    id: i32,
    title: String,
    body: String,
    published: bool,
}

#[derive(Deserialize)]
struct Config {
    name: String,
    age: u8,
}

#[get("/config")]
fn get_config(config: &State<Config>) -> String {
    format!(
      "Hello, {}! You are {} years old.", config.name, config.age
   )
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/random")]
fn get_random_blog_post() -> Json<BlogPost> {
    Json(
        BlogPost {
            id: 1,
            title: "My first post".to_string(),
            body: "This is my first post".to_string(),
            published: true,
        }
    )
}

#[get("/<id>")]
async fn get_blog_post(connection:Db, id: i32) -> Json<BlogPost> {
    connection
        .run( move |c| blog_posts::table.filter(blog_posts::id.eq(id)).first(c))
        .await
        .map(Json)
        .expect("Faild to fetch blog post")
}

#[get("/")]
async fn get_all_blog_posts(connection: Db) -> Json<Vec<BlogPost>> {
    connection
        .run(|c| blog_posts::table.load(c))
        .await
        .map(Json)
        .expect("Failed to fetch blog posts")
}


#[post("/", data = "<blog_post>")]
async fn create_blog_post(connection: Db, blog_post: Json<BlogPost>) -> Json<BlogPost> {
    connection
        .run(move |c| {
            diesel::insert_into(blog_posts::table)
                .values(&blog_post.into_inner())
                .get_result(c)
        })
        .await
        .map(Json)
        .expect("boo")
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();

    rocket
        .attach(Db::fairing())
        .attach(AdHoc::config::<Config>())
        .mount("/", routes![index, get_config])
        .mount("/blog-posts", 
            routes![
                get_random_blog_post,
                get_all_blog_posts,
                get_blog_post,
                create_blog_post
            ]
        )
}
