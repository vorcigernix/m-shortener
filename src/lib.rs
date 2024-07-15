use anyhow::Result;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response, Router},
    http_component,
    sqlite::{Connection, Value},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Link {
    name: String,
    #[serde(default = "rand_string")]
    short_url: String,
    url: String,
}

fn rand_string() -> String {
    return rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
}

#[http_component]
fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut router = Router::default();
    router.get("/:short", get_url_by_short);
    router.post("/add", post_url);
    router.delete("/:short", delete_url);
    Ok(router.handle(req))
}

fn get_url_by_short(_: Request, params: Params) -> Result<impl IntoResponse> {
    let Some(id) = params.get("short") else {
        return Ok(Response::new(400, "no identifier provided"));
    };
    let connection: Connection = Connection::open_default()?;

    let rowset = connection.execute(
        "SELECT full_url FROM links WHERE short = ?",
        &[Value::Text(id.to_string())],
    )?;    
    let row = rowset.rows().next();
    return match row {
        Some(row) => {
            let url = row.get::<&str>("full_url").unwrap().to_string();
            Ok(Response::new(200, url))
        }
        None => Ok(Response::new(404, "not found")),
    };
}

fn post_url(req: Request, _: Params) -> Result<impl IntoResponse> {
    let link = match serde_json::from_slice::<Link>(req.body()) {
        Ok(link) => link,
        Err(error) => return Ok(Response::new(500, error.to_string())),
    };

    let connection = Connection::open_default()?;
    connection.execute(
        "INSERT INTO links (name, short, full_url) VALUES (?, ?, ?)",
        &[
            Value::Text(link.name),
            Value::Text(link.short_url),
            Value::Text(link.url),
        ],
    )?;
    Ok(Response::new(200, "ok"))
}

fn delete_url(_: Request, params: Params) -> Result<impl IntoResponse> {
    let Some(id) = params.get("short") else {
        return Ok(Response::new(400, "no identifier provided"));
    };
    let connection = Connection::open_default()?;
    connection.execute(
        "DELETE FROM links WHERE short = ?",
        &[Value::Text(id.to_string())],
    )?;
    Ok(Response::new(200, "ok"))
}
