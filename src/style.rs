use leptos::prelude::*;

const CSS: &str = include_str!("../style.css");

pub fn stylesheet() -> impl IntoView {
    view! { <style>{CSS}</style> }
}
