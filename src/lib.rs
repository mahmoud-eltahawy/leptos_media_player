//! # leptos_media_player
//!
//! A small, styleable media-player component for [Leptos](https://leptos.dev),
//! packaged with a YouTube-style watch page and a scrollable playlist column.
//!
//! ## What's in the box
//!
//! | Item            | What it does                                                                                                                  |
//! |-----------------|-------------------------------------------------------------------------------------------------------------------------------|
//! | [`MediaPlayer`] | A `<video>` (or audio) surface with play/pause, seek, volume, mute, fullscreen, and prev/next controls.                        |
//! | [`Playlist`]    | A scrollable list of [`MediaItem`]s with optional edit-mode (rename / delete / reorder) and download links.                    |
//! | [`WatchPage`]   | A convenience wrapper that lays the two out YouTube-style: player on the left, playlist on the right. Stacks vertically on phones. |
//! | [`MediaItem`]   | The playlist entry type: `id`, `title`, optional `subtitle`, `src`, optional `artwork`.                                        |
//!
//! The three components are independent — use [`WatchPage`] when you want the
//! default composition, or wire [`MediaPlayer`] and [`Playlist`] together
//! yourself when you need a different layout (playlist above the player,
//! playlist in a modal, two players, …). They coordinate only through the
//! `items` and `current_idx` signals you hand them.
//!
//! ## Styling
//!
//! All styles live in the crate's embedded stylesheet (injected automatically
//! by each component — there's no external CSS file to wire up). Theme by
//! overriding the CSS custom properties on `:root` or any ancestor:
//!
//! ```css
//! :root {
//!     --lmp-accent: #f472b6;
//!     --lmp-radius: 0.75rem;
//! }
//! ```
//!
//! See `style.css` in the repository for the full list of variables.
//!
//! ## Responsive behaviour
//!
//! [`WatchPage`] stacks the player above the playlist on phones (`< 768 px`)
//! and switches to a side-by-side layout at `≥ 768 px`. No JavaScript is
//! involved — it's a single CSS media query.
//!
//! ## Usage
//!
//! ### Video playlist
//!
//! ```rust,no_run
//! use leptos::prelude::*;
//! use leptos_media_player::{MediaItem, WatchPage};
//!
//! #[component]
//! pub fn App() -> impl IntoView {
//!     let items = RwSignal::new(vec![
//!         MediaItem::new(1, "Big Buck Bunny", "/media/bbb.mp4")
//!             .with_subtitle("2008 · 9:56"),
//!         MediaItem::new(2, "Sintel", "/media/sintel.mp4")
//!             .with_subtitle("2010 · 14:48"),
//!         MediaItem::new(3, "Tears of Steel", "/media/tos.mp4")
//!             .with_subtitle("2012 · 12:14"),
//!     ]);
//!     let current_idx = RwSignal::new(0usize);
//!     let edit_mode   = RwSignal::new(false);
//!
//!     let items_sig = Signal::derive(move || items.get());
//!
//!     view! {
//!         <WatchPage
//!             items=items_sig
//!             current_idx=current_idx
//!             edit_mode=edit_mode
//!         />
//!     }
//! }
//! ```
//!
//! ### Audio with a default artwork
//!
//! Set `audio = true` and pass a default `artwork` image URL. Each
//! [`MediaItem`] can also carry its own `artwork`, which wins over the
//! component-level fallback. `artwork` is `#[prop(into)]`, so a bare `String`
//! is accepted as well as `Some(...)`.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_media_player::{MediaItem, WatchPage};
//! # #[component]
//! # fn App() -> impl IntoView {
//! # let items = RwSignal::new(Vec::<MediaItem>::new());
//! # let current_idx = RwSignal::new(0usize);
//! # let edit_mode = RwSignal::new(false);
//! # let items_sig = Signal::derive(move || items.get());
//! view! {
//!     <WatchPage
//!         items=items_sig
//!         current_idx=current_idx
//!         edit_mode=edit_mode
//!         audio=true
//!         artwork=Some("/covers/default.jpg".to_string())
//!     />
//! }
//! # }
//! ```
//!
//! ### Editable playlist
//!
//! Pass `on_rename`, `on_delete`, and `on_move` callbacks alongside an
//! `edit_mode` signal; when the signal is `true`, the playlist surfaces
//! per-row affordances. The callbacks are where you mutate your own state —
//! the component only reports the intent.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_media_player::{MediaItem, WatchPage};
//! # #[component]
//! # fn App() -> impl IntoView {
//! let items = RwSignal::new(vec![
//!     MediaItem::new(1, "One", "/1.mp4"),
//!     MediaItem::new(2, "Two", "/2.mp4"),
//! ]);
//! let current_idx = RwSignal::new(0usize);
//! let edit_mode   = RwSignal::new(true);
//!
//! let on_rename = Callback::new(move |(id, new_title): (u64, String)| {
//!     items.update(|list| {
//!         if let Some(item) = list.iter_mut().find(|i| i.id == id) {
//!             item.title = new_title;
//!         }
//!     });
//! });
//! let on_delete = Callback::new(move |id: u64| {
//!     items.update(|list| list.retain(|i| i.id != id));
//! });
//! let on_move = Callback::new(move |(id, up): (u64, bool)| {
//!     items.update(|list| {
//!         if let Some(i) = list.iter().position(|x| x.id == id) {
//!             let j = if up {
//!                 i.saturating_sub(1)
//!             } else {
//!                 (i + 1).min(list.len().saturating_sub(1))
//!             };
//!             if i != j { list.swap(i, j); }
//!         }
//!     });
//! });
//!
//! let items_sig = Signal::derive(move || items.get());
//!
//! view! {
//!     <WatchPage
//!         items=items_sig
//!         current_idx=current_idx
//!         edit_mode=edit_mode
//!         on_rename=on_rename
//!         on_delete=on_delete
//!         on_move=on_move
//!     />
//! }
//! # }
//! ```
//!
//! ### Lower level: player + playlist side by side
//!
//! [`WatchPage`] is a thin composition. If you want a different layout — a
//! custom sidebar, a modal playlist, two players sharing one playlist — use
//! [`MediaPlayer`] and [`Playlist`] directly. They share state purely through
//! the signals you pass in.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_media_player::{MediaItem, MediaPlayer, Playlist};
//! # #[component]
//! # fn App() -> impl IntoView {
//! # let items = RwSignal::new(Vec::<MediaItem>::new());
//! let current_idx = RwSignal::new(0usize);
//! let edit_mode   = RwSignal::new(false);
//!
//! let items_sig = Signal::derive(move || items.get());
//!
//! view! {
//!     // The `lmp-watch` / `lmp-watch-main` classes give you the same
//!     // responsive layout WatchPage uses — feel free to swap in your own.
//!     <div class="lmp-watch">
//!         <div class="lmp-watch-main">
//!             <MediaPlayer items=items_sig current_idx=current_idx/>
//!         </div>
//!         <Playlist
//!             items=items_sig
//!             current_idx=current_idx
//!             edit_mode=edit_mode
//!         />
//!     </div>
//! }
//! # }
//! ```
//!
//! ## Wiring in a Leptos app
//!
//! Add the crate to `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! leptos_media_player = { path = "../leptos_media_player" }
//! ```
//!
//! …and drop [`WatchPage`] anywhere in your app's tree. No global CSS import
//! is needed — the stylesheet is embedded and injected on first render.

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

/// One entry in the playlist.
///
/// `id` is the stable key used for reordering, renaming, and deletion; it
/// should be unique across a playlist. `title` is shown in both the player's
/// control overlay and the playlist row. `subtitle` is optional (a year,
/// duration, artist name, …). `src` is the URL handed to the `<video>`
/// element. `artwork` is an optional cover image URL used by the audio
/// layout; it overrides the component-level `artwork` fallback.
///
/// ```rust,no_run
/// # use leptos_media_player::MediaItem;
/// let item = MediaItem::new(42, "Big Buck Bunny", "/media/bbb.mp4")
///     .with_subtitle("2008 · 9:56");
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaItem {
    /// Stable identifier; must be unique within a playlist.
    pub id: u64,
    /// Primary label shown in the controls and playlist.
    pub title: String,
    /// Optional secondary line shown under the title in the playlist.
    pub subtitle: Option<String>,
    /// Media URL (`mp4`, `webm`, `mp3`, stream, …) passed to `<video src>`.
    pub src: String,
    /// Optional cover image URL, used by the audio layout.
    pub artwork: Option<String>,
}

impl MediaItem {
    /// Build a minimal item with just an `id`, `title`, and `src`.
    ///
    /// ```rust,no_run
    /// # use leptos_media_player::MediaItem;
    /// let item = MediaItem::new(1, "Intro", "/media/intro.mp4");
    /// ```
    pub fn new(id: u64, title: impl Into<String>, src: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            subtitle: None,
            src: src.into(),
            artwork: None,
        }
    }

    /// Attach a subtitle line (builder style).
    ///
    /// ```rust,no_run
    /// # use leptos_media_player::MediaItem;
    /// let item = MediaItem::new(1, "Intro", "/media/intro.mp4")
    ///     .with_subtitle("00:42");
    /// ```
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
}

// ─── Context plumbing ─────────────────────────────────────────────────────
// Internal helper used to bundle several signals / callbacks into a single
// `provide_context` slot. Not part of the public API.

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
