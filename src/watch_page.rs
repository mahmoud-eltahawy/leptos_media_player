use leptos::prelude::*;

use crate::{MediaItem, MediaPlayer, Playlist};

/// A ready-made watch page: [`MediaPlayer`] and [`Playlist`] composed in a
/// YouTube-style layout — player on the left, playlist column on the right,
/// stacking vertically on narrow screens.
///
/// This is a convenience wrapper. It takes the union of the two primitives'
/// props and threads them through. If you want a different composition
/// (playlist above the player, playlist in a modal, two players side by
/// side) use [`MediaPlayer`] and [`Playlist`] directly — they are not
/// coupled.
///
/// The playlist is hidden automatically when there is only one item, since
/// an "up next" column with a single entry is noise.
#[component]
pub fn WatchPage(
    items: Signal<Vec<MediaItem>>,
    current_idx: RwSignal<usize>,
    #[prop(into)] edit_mode: Signal<bool>,
    #[prop(default = false)] audio: bool,
    #[prop(default = None)] artwork: Option<String>,
    #[prop(default = None)] playlist_title: Option<String>,
    #[prop(default = None)] on_rename: Option<Callback<(u64, String)>>,
    #[prop(default = None)] on_delete: Option<Callback<u64>>,
    #[prop(default = None)] on_move: Option<Callback<(u64, bool)>>,
    #[prop(default = true)] show_download: bool,
) -> impl IntoView {
    let show_playlist = Signal::derive(move || items.get().len() > 1);
    let title_for_playlist = playlist_title.clone();

    view! {
        <div class="lmp-watch">
            <div class="lmp-watch-main">
                <MediaPlayer
                    items=items
                    current_idx=current_idx
                    audio=audio
                    artwork=artwork
                />
            </div>
            <Show when=move || show_playlist.get()>
                <Playlist
                    items=items
                    current_idx=current_idx
                    edit_mode=edit_mode
                    title=title_for_playlist.clone()
                    on_rename=on_rename
                    on_delete=on_delete
                    on_move=on_move
                    show_download=show_download
                />
            </Show>
        </div>
    }
}
