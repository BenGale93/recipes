#![warn(clippy::all, clippy::nursery)]
#![allow(dead_code)]
use std::{net::SocketAddr, sync::Arc};

use axum::{extract::State, response::Html, routing::get, Router};
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};

const HOME_TEMPLATE: &str = r#"
<body style='padding: 30px;'>
<div>
    <ul>
    {% for recipe in recipes %}
        <li><a href='#{{recipe.anchor}}'>{{recipe.name}}</a></li>
    {% endfor %}
    </ul>
</div>
{% for recipe in recipes %}
    <div id='{{recipe.anchor}}'>
        <h1>{{recipe.name}}</h1>
        <ul>
            {% for item in recipe.ingredients %}
            <li>{{ item }}</li>
            {% endfor %}
        </ul>
        <p style='white-space: pre-line;'>
            {{recipe.recipe}}
        </p>
    </div>
{% endfor %}
</body>
"#;

#[derive(Deserialize)]
struct DeserializeRecipe {
    name: String,
    ingredients: Vec<String>,
    recipe: String,
}

impl From<DeserializeRecipe> for Recipe {
    fn from(tmp: DeserializeRecipe) -> Self {
        Self::new(tmp.name, tmp.ingredients, tmp.recipe)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(from = "DeserializeRecipe")]
struct Recipe {
    name: String,
    ingredients: Vec<String>,
    recipe: String,
    #[serde(skip_serializing)]
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
    pub recipes: Arc<Vec<Recipe>>,
    pub env: Arc<Environment<'a>>,
}

#[tokio::main]
async fn main() {
    let recipes_string = std::fs::read_to_string("recipes.yaml").expect("Expected file to exist.");
    let recipes_yaml = serde_yaml::from_str(&recipes_string).expect("Expected to parse YAML.");
    let recipes: Arc<Vec<Recipe>> = Arc::new(recipes_yaml);

    let mut env = Environment::new();
    env.set_debug(true);
    env.add_template("home", HOME_TEMPLATE).unwrap();

    let app_state = Arc::new(AppState {
        recipes,
        env: Arc::new(env),
    });

    let app = Router::new()
        .route("/", get(display_recipes))
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
        .render(context!(recipes => app_state.recipes.to_vec()))
        .unwrap();

    Html(r)
}
