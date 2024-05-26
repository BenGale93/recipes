use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Form,
};
use minijinja::context;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Deserialize)]
pub struct FormRecipe {
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
pub struct Recipe {
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

#[allow(clippy::unnecessary_to_owned)]
pub async fn display_recipes(State(app_state): State<Arc<AppState<'_>>>) -> Html<String> {
    let template = app_state.env.get_template("home").unwrap();

    let r = template
        .render(context!(recipes => app_state.recipes.lock().await.to_vec()))
        .unwrap();

    Html(r)
}

#[allow(clippy::unnecessary_to_owned)]
pub async fn new_recipe(State(app_state): State<Arc<AppState<'_>>>) -> Html<String> {
    let template = app_state.env.get_template("new").unwrap();

    let r = template.render(context!()).unwrap();

    Html(r)
}

#[allow(clippy::unnecessary_to_owned)]
pub async fn create_recipe(
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
