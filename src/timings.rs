use std::sync::Arc;

use axum::{extract::State, response::Html, Form};
use minijinja::context;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Deserialize)]
pub struct FormEnd {
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
pub struct Timings {
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

pub async fn roast_timings(State(app_state): State<Arc<AppState<'_>>>) -> Html<String> {
    let template = app_state.env.get_template("roast").unwrap();

    let r = template
        .render(context!(end => app_state.timings.lock().await.convert_end()))
        .unwrap();

    Html(r)
}

pub async fn compute_timings(
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
