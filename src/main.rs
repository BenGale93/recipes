#![warn(clippy::all, clippy::nursery)]
use axum::extract::State;
use axum::{response::Html, routing::get, Router};
use serde::Deserialize;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
struct Recipe {
    name: String,
    ingredients: Vec<String>,
    recipe: String,
}

impl Recipe {
    fn html_link(&self) -> String {
        let href = format!("#{}", self.name.replace(' ', "-"));
        format!("<li><a href='{}'>{}</a></li>", href, self.name)
    }

    fn to_html(&self) -> String {
        let anchor = self.name.replace(' ', "-");
        format!(
            "<div id='{}'><h1>{}</h1><p>{}</p><p>{}</p></div>",
            anchor,
            self.name,
            self.ingredients.join("<br>"),
            self.recipe.replace('\n', "<br>")
        )
    }
}

fn create_links(recipes: &[Recipe]) -> String {
    let links: Vec<String> = recipes.iter().map(|r| r.html_link()).collect();
    let links = links.join("");
    format!("<div><ul>{}</ul></div>", links)
}

fn create_bodies(recipes: &[Recipe]) -> String {
    let bodies: Vec<String> = recipes.iter().map(|r| r.to_html()).collect();
    let bodies = bodies.join("");
    format!("<div>{}</div>", bodies)
}

#[tokio::main]
async fn main() {
    let recipes_string = std::fs::read_to_string("recipes.yaml").expect("Expected file to exist.");
    let recipes_yaml = serde_yaml::from_str(&recipes_string).expect("Expected to parse YAML.");
    let recipes: Arc<Vec<Recipe>> = Arc::new(recipes_yaml);
    let app = Router::new()
        .route("/", get(display_recipes))
        .with_state(recipes);

    // run it
    #[cfg(not(debug_assertions))]
    let addr = SocketAddr::from(([192, 168, 1, 92], 3000));
    #[cfg(debug_assertions)]
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn display_recipes(State(recipes): State<Arc<Vec<Recipe>>>) -> Html<String> {
    let links = create_links(&recipes);
    let bodies = create_bodies(&recipes);

    Html(format!(
        "<body style='padding: 30px;'>{}{}</body>",
        links, bodies
    ))
}
