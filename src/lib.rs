use leptos::prelude::*;
use serde::{Deserialize, Serialize};

mod icons;
mod player;
mod playlist;
mod style;
mod watch_page;

pub use player::MediaPlayer;
pub use playlist::Playlist;
pub use watch_page::WatchPage;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: u64,
    pub title: String,
    pub subtitle: Option<String>,
    pub src: String,
    pub artwork: Option<String>,
}

impl MediaItem {
    pub fn new(id: u64, title: impl Into<String>, src: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            subtitle: None,
            src: src.into(),
            artwork: None,
        }
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
}

#[derive(Clone, Copy)]
struct Bundle<T>(T);

trait ContextBundle: Clone + Send + Sync + 'static {
    fn provide(self) {
        provide_context(Bundle(self));
    }
    fn expect() -> Self {
        expect_context::<Bundle<Self>>().0
    }
}
