use std::str;

use serde::{Deserialize, Serialize};
use worker::*;
use worker::wasm_bindgen::JsValue;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get("/", |_, _| Response::ok("OK"))
        .get_async("/redirect/:key", |_, ctx| async move {
            if let Some(key) = ctx.param("key") {
                let db: D1Database = ctx.env.d1("DB")?;

                let statement = db
                    .prepare("SELECT long_url FROM links WHERE key =?")
                    .bind(&[JsValue::from(key)])?;

                let results: Option<String> = statement.first(Some("long_url")).await?;

                return match results {
                    None => {
                        Response::error("URL Unknown", 404)
                    }
                    Some(url) => {
                        let d1result = increment_link_clicks(key, &db).await?;
                        if !d1result.success() {
                            console_log!("Error while updating clicks for link {key:}")
                        }
                        Response::redirect(Url::parse(url.as_str())?)
                    },
                };
            }
            Response::error("Key Missing", 400)
        })
        .get_async("/info/:key", |_, ctx| async move {
            if let Some(key) = ctx.param("key") {
                let db: D1Database = ctx.env.d1("DB")?;

                let statement = db
                    .prepare("SELECT * FROM links WHERE key = ?")
                    .bind(&[JsValue::from(key)])?;

                let results = statement.all().await?;

                let result = results.results::<Link>()?;

                return match result.get(0) {
                    None => {
                        return Response::error("Key not found", 404);
                    }
                    Some(url) => Response::ok(serde_json::to_string(url)?),
                };
            }
            Response::error("Key Missing", 400)
        })
        .put_async("/", |mut req, ctx| async move {
            let buf = req.bytes().await?;

            if let Ok(data) = str::from_utf8(buf.as_slice()) {
                let result: Link = serde_json::from_str(data)?;

                let db: D1Database = ctx.env.d1("DB")?;

                let statement = db
                    .prepare("INSERT INTO links(`key`, `long_url`, `clicks`) VALUES (?,?,?)")
                    .bind(&[JsValue::from(result.key.as_str()), JsValue::from(result.long_url.as_str()), JsValue::from(result.clicks.to_string().as_str())])?;

                return match statement.run().await {
                    Ok(_) => Response::ok("OK"),
                    Err(err) => Response::error(format!("Database error! {err:?}"), 500)
                }
            }

            Response::error("Invalid data send", 500)
        })
        .patch_async("/", |mut req, ctx| async move {
            let buf = req.bytes().await?;

            if let Ok(data) = str::from_utf8(buf.as_slice()) {
                let result: Link = serde_json::from_str(data)?;

                let db: D1Database = ctx.env.d1("DB")?;

                let statement = db
                    .prepare("INSERT INTO links(`key`, `long_url`, `clicks`) VALUES (?,?,?) ON CONFLICT(`key`) DO UPDATE SET `long_url` = ?, `clicks` = ?")
                    .bind(&[JsValue::from(result.key.as_str()), JsValue::from(result.long_url.as_str()), JsValue::from(result.clicks.to_string().as_str()), JsValue::from(result.long_url.as_str()), JsValue::from(result.clicks.to_string().as_str())])?;

                return match statement.run().await {
                    Ok(_) => Response::ok("OK"),
                    Err(err) => Response::error(format!("Database error! {err:?}"), 500)
                }
            }

            Response::error("Invalid data send", 500)
        })
        .delete_async("/:key", |_, ctx| async move {
            if let Some(key) = ctx.param("key") {
                let db: D1Database = ctx.env.d1("DB")?;

                let statement = db
                    .prepare("DELETE FROM links WHERE key = ? ")
                    .bind(&[JsValue::from(key)])?;

                let results = statement.run().await?;
                return if results.success() {
                    Response::ok("Link deleted")
                } else {
                    Response::error("Can not delete link", 500)
                };
            }
            Response::error("Key Missing", 400)
        })
        .run(req, env)
        .await
}

async fn increment_link_clicks(key: &String, database: &D1Database) -> Result<D1Result> {
    let statement = database
        .prepare("UPDATE links SET `clicks`=`clicks` + 1 WHERE `key` = ?")
        .bind(&[JsValue::from(key)])?;

    statement.run().await
}

#[derive(Debug, Deserialize, Serialize)]
struct Link {
    key: String,
    long_url: String,
    clicks: usize,
}
