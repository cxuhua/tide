use http_types::Mime;
use rust_embed::RustEmbed;
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;
use tide::log;
use tide::prelude::*;
use tide::Body;
use tide::{Middleware, Next, Request};
// use tide_rustls::rustls::Session;
// use tide_rustls::TlsListener;

#[async_std::test]
async fn successfully_retrieve_request_cookie() {
    use indexmap::IndexMap;
    let mut process_table = IndexMap::new();
    process_table.entry("vim0").or_insert(10);
    process_table.entry("vim1").or_insert(11);
    println!("{}", process_table["vim0"]);
    println!("{}", process_table[1]);
}

async fn run_redis() -> redis::RedisResult<redis::aio::Connection> {
    use redis::ErrorKind;
    match redis::Client::open("redis://192.168.3.84/") {
        Ok(client) => Ok(client.get_async_connection().await?),
        Err(_) => Err((ErrorKind::ResponseError, "err", "aaa".into()).into()),
    }
}

#[async_std::test]
async fn test_redis() {
    use redis::AsyncCommands;
    let mut con = run_redis().await.unwrap();
    let _: () = con.set("aa", -123).await.unwrap();
    let size: i64 = con.get("aa").await.unwrap();
    println!("{}", size);
}
#[async_std::test]
async fn test_mongodb() {
    use mongodb::bson::{doc, oid::ObjectId, Document};
    use mongodb::{options::ClientOptions, options::FindOptions, Client};
    let mut client_options = ClientOptions::parse("mongodb://192.168.3.84:27017/?replicaSet=rs")
        .await
        .unwrap();
    // Manually set an option.
    client_options.app_name = Some("test app".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();

    // List the names of the databases in that deployment.
    for db_name in client.list_database_names(None, None).await.unwrap() {
        println!("{}", db_name);
    }
    let db = client.database("mydb");
    // let collection = db.collection::<Document>("books");

    // let docs = vec![
    //     doc! { "title": "1984", "author": "George Orwell" },
    //     doc! { "title": "Animal Farm", "author": "George Orwell" },
    //     doc! { "title": "The Great Gatsby", "author": "F. Scott Fitzgerald" },
    // ];

    // // Insert some documents into the "mydb.books" collection.
    // collection.insert_many(docs, None).await.unwrap();

    use futures::stream::TryStreamExt;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Book {
        #[serde(rename = "_id")]
        id: ObjectId,
        title: String,
        author: String,
    }

    let typed_collection = db.collection::<Book>("books");

    let filter = doc! { "author": "George Orwell" };
    let find_options = FindOptions::builder().sort(doc! { "title": 1 }).build();
    let mut cursor = typed_collection.find(filter, find_options).await.unwrap();

    // Iterate over the results of the cursor.
    while let Some(book) = cursor.try_next().await.unwrap() {
        println!("title: {:#?}", book);
    }
}

#[derive(RustEmbed)]
#[folder = "public"]
struct Asset;

struct Middle {}

use infer::Infer;

struct Animal {
    name: String,
    legs: u8,
    infer: Infer,
}

#[tide::utils::async_trait]
impl Middleware<Arc<Animal>> for Middle {
    async fn handle(&self, req: Request<Arc<Animal>>, next: Next<'_, Arc<Animal>>) -> tide::Result {
        println!("{} - {}", req.state().name, req.state().legs);
        Ok(next.run(req).await)
        // Ok(StatusCode::Ok.into())
    }
    fn name(&self) -> &str {
        "test middle"
    }
}

use tide::sse;

#[async_std::main]
async fn main() -> tide::Result<()> {
    // let index_html = Asset::get("qungonghuo.jpg").unwrap();
    // println!("{:?}", index_html.data);

    std::env::set_var("TIDE_CERT_PATH", "d:\\keys\\server.crt");
    std::env::set_var("TIDE_KEY_PATH", "d:\\keys\\server.key");
    log::start();

    let mut infer = Infer::new();
    infer.add("txt", "txt", |v| true);
    let mut app = tide::with_state(Arc::new(Animal {
        name: "hello world".into(),
        legs: 8,
        infer: infer,
    }));
    app.at("/*").with(Middle {}).get(order_shoes);
    // app.with(Middle {});
    // app.listen(
    //     TlsListener::build()
    //         .addrs("0.0.0.0:4433")
    //         .cert(std::env::var("TIDE_CERT_PATH").unwrap())
    //         .key(std::env::var("TIDE_KEY_PATH").unwrap()),
    // )
    // .await?;
    app.listen("0.0.0.0:8080").await?;
    Ok(())
}

// 获取文件类型Mime
async fn get_mime(bytes: &[u8], path: &str, default: &str) -> Mime {
    Mime::sniff(bytes).map_or_else(
        |_| {
            let path = Path::new(path);
            let ext = path
                .extension()
                .map_or(default, |v| v.to_str().map_or(default, |v| v));
            Mime::from_extension(ext).map_or(http_types::mime::BYTE_STREAM, |v| v)
        },
        |v| v,
    )
}

use async_std::io::Cursor;
use http_types::StatusCode;
use std::convert::From;
// 读取打包静态文件数据
async fn order_shoes(req: Request<Arc<Animal>>) -> tide::Result {
    let path = req.url().path();
    if path.is_empty() {
        return Ok(StatusCode::BadRequest.into());
    }
    match Asset::get(&path[1..]) {
        Some(content) => {
            let len = content.data.len();
            if len == 0 {
                return Ok(StatusCode::NoContent.into());
            }
            let cursor = Cursor::new(content.data);
            let mime = get_mime(cursor.get_ref(), path, "jpg").await;
            let mut body = Body::from_reader(cursor, Some(len));
            body.set_mime(mime);
            Ok(body.into())
        }
        None => Ok(StatusCode::NotFound.into()),
    }
}
