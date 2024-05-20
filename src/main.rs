#![warn(clippy::all, clippy::nursery)]
#![allow(dead_code)]
use std::{fmt::Debug, fs::File, net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use minijinja::{context, Environment};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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
        .route("/", get(display_recipes))
        .route("/new", get(new_recipe))
        .route("/new_recipe", post(create_recipe))
        .route("/roast", get(roast_timings))
        .route("/roast", post(compute_timings))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
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

#[derive(Deserialize)]
struct FormEnd {
    end: String,
}

impl TryFrom<FormEnd> for chrono::NaiveTime {
    type Error = ();

    fn try_from(value: FormEnd) -> Result<Self, Self::Error> {
        Ok(Self::parse_from_str(&value.end, "%H:%M").unwrap())
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Step {
    step: String,
    offset: i64,
}

#[derive(Deserialize, Serialize, Debug)]
struct Timings {
    end: chrono::NaiveTime,
    steps: Vec<Step>,
}

impl Timings {
    pub fn times(&self) -> Vec<(String, String)> {
        self.steps
            .iter()
            .map(|s| {
                let duration = chrono::Duration::minutes(s.offset);
                let time = self.end + duration;
                (s.step.to_owned(), Self::convert_time_to_string(time))
            })
            .collect()
    }

    pub fn convert_end(&self) -> String {
        Self::convert_time_to_string(self.end)
    }

    fn convert_time_to_string(time: chrono::NaiveTime) -> String {
        time.format("%H:%M").to_string()
    }
}

async fn roast_timings(State(app_state): State<Arc<AppState<'_>>>) -> Html<String> {
    let template = app_state.env.get_template("roast").unwrap();

    let r = template
        .render(context!(end => app_state.timings.lock().await.convert_end()))
        .unwrap();

    Html(r)
}

async fn compute_timings(
    State(app_state): State<Arc<AppState<'_>>>,
    Form(form_timings): Form<FormEnd>,
) -> Html<String> {
    app_state.timings.lock().await.end = form_timings.try_into().unwrap();

    let template = app_state.env.get_template("steps").unwrap();

    let r = {
        template
            .render(context!(steps => app_state.timings.lock().await.times()))
            .unwrap()
    };

    Html(r)
}
