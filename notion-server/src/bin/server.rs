#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

#[get("/")]
fn index() -> &'static str {
    "Welcome to the Seamulator"
}

#[get("/beacon")]
fn get_beacon() -> &'static str {
    "GET /beacon: unclear what this returns, probably general beacon state, but for now not much there"
}

#[get("/beacon/execution_environments")]
fn get_execution_environments() -> &'static str {
    "GET /beacon/execution_environments: should return a list of EEs"
}

#[get("/beacon/execution_environments/<ee_index>")]
fn get_specific_execution_environment(ee_index: u32) -> String {
    format!("GET /beacon/execution_environments/{}: should return ee at index {}", ee_index, ee_index)
}

#[get("/shards")]
fn get_shards() -> &'static str {
    "GET /shards"
}

#[get("/shards/<shard_id>")]
fn get_specific_shard(shard_id: u32) -> String {
    format!("GET /shards/{} -> maybe have this return EE state for this shard?", shard_id)
}

#[get("/shards/<shard_id>/blocks")]
fn get_blocks_for_shard(shard_id: u32) -> String {
    format!("GET /shards/{}/blocks", shard_id)
}

#[derive(Serialize)]
struct EeIndexResponse {
    ee_index: i32,
}
#[post("/beacon/execution_environments", data = "<ee_args>")]
fn add_new_execution_environment(ee_args: Json<mylib::eth_magic::CreateEeArgs>) -> Json<EeIndexResponse> {
    println!("server received struct: {:?}", ee_args);
    Json(EeIndexResponse {
        ee_index: 0,
    })
}

#[post("/shards")]
fn create_new_shard_chain() -> &'static str {
    "POST /shards: Eventually this returns JSON with the index of the shard chain that was created"
}

#[post("/shards/<shard_id>/blocks")]
fn append_shard_block(shard_id: u32) -> String {
    format!("POST /shards/{}/blocks", shard_id)
}

fn main() {
    let routes = routes![
        index,
        get_beacon,
        get_execution_environments,
        get_specific_execution_environment,
        get_shards,
        get_specific_shard,
        get_blocks_for_shard,
        add_new_execution_environment,
        create_new_shard_chain,
        append_shard_block,
    ];
    rocket::ignite().mount("/", routes).launch();
}