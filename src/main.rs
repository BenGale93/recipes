#![warn(clippy::all, clippy::nursery)]
#![allow(dead_code)]
use std::{fs::File, net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use minijinja::Environment;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;

mod home;
mod template;
mod timings;

use home::Recipe;
use timings::Timings;

#[derive(Clone)]
struct AppState<'a> {
    pub recipes: Arc<Mutex<Vec<Recipe>>>,
    pub timings: Arc<Mutex<Timings>>,
    pub env: Arc<Environment<'a>>,
}

fn read_config<T>(path: &str) -> anyhow::Result<Arc<Mutex<T>>>
where
    T: DeserializeOwned,
{
    let file = File::open(path)?;
    let config = serde_yaml::from_reader(file)?;
    Ok(Arc::new(Mutex::new(config)))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let recipes: Arc<Mutex<Vec<Recipe>>> = read_config("recipes.yaml")?;
    let timings: Arc<Mutex<Timings>> = read_config("timings.yaml")?;

    let mut env = Environment::new();
    env.set_debug(true);
    env.add_template("base", template::BASE_TEMPLATE)?;
    env.add_template("home", template::HOME_TEMPLATE)?;
    env.add_template("new", template::NEW_TEMPLATE)?;
    env.add_template("roast", template::ROAST_TEMPLATE)?;
    env.add_template("steps", template::STEPS_TEMPLATE)?;

    let app_state = Arc::new(AppState {
        recipes,
        timings,
        env: Arc::new(env),
    });

    let app = Router::new()
        .route("/", get(home::display_recipes))
        .route("/new", get(home::new_recipe))
        .route("/new_recipe", post(home::create_recipe))
        .route("/roast", get(timings::roast_timings))
        .route("/roast", post(timings::compute_timings))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
