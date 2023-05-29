#![warn(clippy::all, clippy::nursery)]
#![allow(dead_code)]
use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

mod template;

#[derive(Deserialize)]
struct FormRecipe {
    name: String,
    ingredients: String,
    recipe: String,
}

impl From<FormRecipe> for Recipe {
    fn from(tmp: FormRecipe) -> Self {
        let ingredients: Vec<String> = tmp.ingredients.split('\n').map(String::from).collect();
        Self::new(tmp.name, ingredients, tmp.recipe)
    }
}

#[derive(Deserialize, Serialize)]
struct StoredRecipe {
    name: String,
    ingredients: Vec<String>,
    recipe: String,
}

impl StoredRecipe {
    fn new(name: String, ingredients: Vec<String>, recipe: String) -> Self {
        Self {
            name,
            ingredients,
            recipe,
        }
    }
}

impl From<StoredRecipe> for Recipe {
    fn from(tmp: StoredRecipe) -> Self {
        Self::new(tmp.name, tmp.ingredients, tmp.recipe)
    }
}

impl From<Recipe> for StoredRecipe {
    fn from(tmp: Recipe) -> Self {
        Self::new(tmp.name, tmp.ingredients, tmp.recipe)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(from = "StoredRecipe")]
struct Recipe {
    name: String,
    ingredients: Vec<String>,
    recipe: String,
    anchor: String,
}

impl Recipe {
    fn new(name: String, ingredients: Vec<String>, recipe: String) -> Self {
        let anchor = name.replace(' ', "-");
        Self {
            name,
            ingredients,
            recipe,
            anchor,
        }
    }
}

#[derive(Clone)]
struct AppState<'a> {
    pub recipes: Arc<Mutex<Vec<Recipe>>>,
    pub env: Arc<Environment<'a>>,
}

#[tokio::main]
async fn main() {
    let recipes_string = std::fs::read_to_string("recipes.yaml").expect("Expected file to exist.");
    let recipes_yaml = serde_yaml::from_str(&recipes_string).expect("Expected to parse YAML.");
    let recipes: Arc<Mutex<Vec<Recipe>>> = Arc::new(Mutex::new(recipes_yaml));

    let mut env = Environment::new();
    env.set_debug(true);
    env.add_template("base", template::BASE_TEMPLATE).unwrap();
    env.add_template("home", template::HOME_TEMPLATE).unwrap();
    env.add_template("new", template::NEW_TEMPLATE).unwrap();

    let app_state = Arc::new(AppState {
        recipes,
        env: Arc::new(env),
    });

    let app = Router::new()
        .route("/", get(display_recipes))
        .route("/new", get(new_recipe))
        .route("/new_recipe", post(create_recipe))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[allow(clippy::unnecessary_to_owned)]
async fn display_recipes(State(app_state): State<Arc<AppState<'_>>>) -> Html<String> {
    let template = app_state.env.get_template("home").unwrap();

    let r = template
        .render(context!(recipes => app_state.recipes.lock().await.to_vec()))
        .unwrap();

    Html(r)
}

#[allow(clippy::unnecessary_to_owned)]
async fn new_recipe(State(app_state): State<Arc<AppState<'_>>>) -> Html<String> {
    let template = app_state.env.get_template("new").unwrap();

    let r = template.render(context!()).unwrap();

    Html(r)
}

#[allow(clippy::unnecessary_to_owned)]
async fn create_recipe(
    State(app_state): State<Arc<AppState<'_>>>,
    Form(form_recipe): Form<FormRecipe>,
) -> impl IntoResponse {
    app_state.recipes.lock().await.push(form_recipe.into());

    let stored_recipes: Vec<_> = app_state.recipes.lock().await.to_vec();
    let stored_recipes: Vec<_> = stored_recipes.into_iter().map(StoredRecipe::from).collect();

    let Ok(recipes_yaml) = serde_yaml::to_string(&stored_recipes) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create recipe");
    };
    std::fs::write("recipes.yaml", recipes_yaml).unwrap();

    (StatusCode::CREATED, "Recipe created")
}
