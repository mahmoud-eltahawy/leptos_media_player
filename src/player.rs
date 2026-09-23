use leptos::wasm_bindgen::JsCast;
use leptos::{either::Either, ev::fullscreenchange};
use leptos::{html, prelude::*};
use leptos_use::{UseTimeoutFnReturn, use_document, use_event_listener, use_timeout_fn};
use web_sys::{HtmlInputElement, MouseEvent};

use crate::icons::{
    FullscreenExitIcon, FullscreenIcon, MuteIcon, NextPageIcon, PauseIcon, PlayIcon, PrevPageIcon,
    VolumeIcon,
};
use crate::style::stylesheet;
use crate::{ContextBundle, MediaItem};

// ─── Context bundles ──────────────────────────────────────────────────────

impl ContextBundle for PlayerSignals {}
impl ContextBundle for PlayerDerived {}
impl ContextBundle for PlayerHandlers {}
impl ContextBundle for PlayerNav {}

// ─── Signal bundle ────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct PlayerSignals {
    playing: RwSignal<bool>,
    current_time: RwSignal<f64>,
    duration: RwSignal<f64>,
    volume: RwSignal<f64>,
    last_volume: RwSignal<f64>,
    muted: RwSignal<bool>,
    fullscreen: RwSignal<bool>,
    controls_visible: RwSignal<bool>,
    play_after_load: RwSignal<bool>,
}

impl PlayerSignals {
    fn new() -> Self {
        Self {
            playing: RwSignal::new(false),
            current_time: RwSignal::new(0.0),
            duration: RwSignal::new(0.0),
            volume: RwSignal::new(1.0),
            last_volume: RwSignal::new(1.0),
            muted: RwSignal::new(false),
            fullscreen: RwSignal::new(false),
            controls_visible: RwSignal::new(true),
            play_after_load: RwSignal::new(false),
        }
    }
}

// ─── Derived signals ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct PlayerDerived {
    current_src: Signal<String>,
    current_title: Signal<String>,
    current_artwork: Signal<Option<String>>,
    has_prev: Signal<bool>,
    has_next: Signal<bool>,
    can_navigate: Signal<bool>,
}

impl PlayerDerived {
    fn build(
        items: Signal<Vec<MediaItem>>,
        current_idx: RwSignal<usize>,
        artwork_fallback: Option<String>,
    ) -> Self {
        let current_item = Memo::new(move |_| items.get().get(current_idx.get()).cloned());

        let current_src =
            Signal::derive(move || current_item.get().map(|i| i.src).unwrap_or_default());
        let current_title =
            Signal::derive(move || current_item.get().map(|i| i.title).unwrap_or_default());
        let current_artwork = {
            let fallback = artwork_fallback;
            Signal::derive(move || {
                current_item
                    .get()
                    .and_then(|i| i.artwork)
                    .or_else(|| fallback.clone())
            })
        };

        let has_prev = Signal::derive(move || current_idx.get() > 0);
        let has_next = Signal::derive(move || current_idx.get() + 1 < items.get().len());
        let can_navigate = Signal::derive(move || items.get().len() > 1);

        Self {
            current_src,
            current_title,
            current_artwork,
            has_prev,
            has_next,
            can_navigate,
        }
    }
}

// ─── Handler bundles ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct PlayerHandlers {
    toggle_play: Callback<MouseEvent>,
    toggle_mute: Callback<MouseEvent>,
    toggle_fullscreen: Callback<MouseEvent>,
    handle_seek: Callback<web_sys::Event>,
    handle_volume: Callback<web_sys::Event>,
}

#[derive(Clone, Copy)]
struct PlayerNav {
    on_prev: Callback<MouseEvent>,
    on_next: Callback<MouseEvent>,
}

// ─── Effects ──────────────────────────────────────────────────────────────

fn install_clamp_effect(items: Signal<Vec<MediaItem>>, current_idx: RwSignal<usize>) {
    Effect::new(move |_| {
        let n = items.get().len();
        if n == 0 {
            current_idx.set(0);
        } else if current_idx.get() >= n {
            current_idx.set(n - 1);
        }
    });
}

fn install_src_reload_effect(
    video_ref: NodeRef<html::Video>,
    signals: PlayerSignals,
    current_src: Signal<String>,
) {
    Effect::new(move || {
        let src = current_src.get();
        if let Some(video) = video_ref.get() {
            video.set_src(&src);
            video.load();
            signals.playing.set(false);
            signals.current_time.set(0.0);
            signals.duration.set(0.0);
            if signals.play_after_load.get_untracked() {
                signals.play_after_load.set(false);
                let _ = video.play();
            }
        }
    });
}

// ─── Handler factories ────────────────────────────────────────────────────

fn make_toggle_play(
    video_ref: NodeRef<html::Video>,
    playing: RwSignal<bool>,
) -> impl Fn(MouseEvent) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            if playing.get() {
                video.pause().ok();
            } else {
                let _ = video.play();
            }
        }
    }
}

fn make_handle_loaded_metadata(
    video_ref: NodeRef<html::Video>,
    duration: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            duration.set(video.duration());
        }
    }
}

fn make_handle_time_update(
    video_ref: NodeRef<html::Video>,
    current_time: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            current_time.set(video.current_time());
        }
    }
}

fn make_handle_seek(
    video_ref: NodeRef<html::Video>,
    current_time: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && let Ok(val) = input.value().parse::<f64>()
            && let Some(video) = video_ref.get()
        {
            video.set_current_time(val);
            current_time.set(val);
        }
    }
}

fn make_handle_volume(
    video_ref: NodeRef<html::Video>,
    volume: RwSignal<f64>,
    muted: RwSignal<bool>,
    last_volume: RwSignal<f64>,
) -> impl Fn(web_sys::Event) + Copy {
    move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && let Ok(val) = input.value().parse::<f64>()
            && let Some(video) = video_ref.get()
        {
            video.set_volume(val);
            video.set_muted(val == 0.0);
            volume.set(val);
            muted.set(val == 0.0);
            if val > 0.0 {
                last_volume.set(val);
            }
        }
    }
}

fn make_toggle_mute(
    video_ref: NodeRef<html::Video>,
    volume: RwSignal<f64>,
    last_volume: RwSignal<f64>,
    muted: RwSignal<bool>,
) -> impl Fn(MouseEvent) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            if muted.get() {
                video.set_muted(false);
                let restore = last_volume.get().max(0.1);
                video.set_volume(restore);
                volume.set(restore);
                muted.set(false);
            } else {
                last_volume.set(volume.get().max(0.1));
                video.set_muted(true);
                muted.set(true);
            }
        }
    }
}

fn make_toggle_fullscreen(video_ref: NodeRef<html::Video>) -> impl Fn(MouseEvent) + Copy {
    move |_| {
        if let Some(video) = video_ref.get() {
            if document().fullscreen_element().is_none() {
                let _ = video.request_fullscreen();
            } else {
                document().exit_fullscreen();
            }
        }
    }
}

// ─── Public component ─────────────────────────────────────────────────────

/// The video (or audio) surface with its controls.
///
/// Pair it with a [`Playlist`](crate::Playlist) that shares the same
/// `items` and `current_idx` signals — or use it standalone.
#[component]
pub fn MediaPlayer(
    items: Signal<Vec<MediaItem>>,
    current_idx: RwSignal<usize>,
    #[prop(default = false)] audio: bool,
    #[prop(default = None)] artwork: Option<String>,
) -> impl IntoView {
    let video_ref = NodeRef::<html::Video>::new();
    let signals = PlayerSignals::new();
    let derived = PlayerDerived::build(items, current_idx, artwork.clone());

    install_clamp_effect(items, current_idx);
    install_src_reload_effect(video_ref, signals, derived.current_src);

    // ── Controls visibility timeout ───────────────────────────────────
    let UseTimeoutFnReturn { start, stop, .. } =
        use_timeout_fn(move |_i: i8| signals.controls_visible.set(false), 3000.);
    let start_hide_timer = {
        let stop = stop.clone();
        move || {
            stop();
            start(3);
        }
    };
    let show_controls = {
        let start_hide_timer = start_hide_timer.clone();
        move || {
            signals.controls_visible.set(true);
            start_hide_timer();
        }
    };
    let toggle_controls = {
        let show_controls = show_controls.clone();
        let stop = stop.clone();
        move || {
            if signals.controls_visible.get() {
                signals.controls_visible.set(false);
                stop();
            } else {
                show_controls();
            }
        }
    };

    // ── Fullscreen listener ───────────────────────────────────────────
    let u_document = use_document();
    let _guard = use_event_listener(u_document.clone(), fullscreenchange, move |_| {
        signals
            .fullscreen
            .set(u_document.fullscreen().is_some_and(|x| x));
    });

    // ── Navigation ────────────────────────────────────────────────────
    let on_next: Callback<MouseEvent> = Callback::new(move |_| {
        if current_idx.get_untracked() + 1 < items.get_untracked().len() {
            current_idx.update(|i| *i += 1);
        }
    });
    let on_prev: Callback<MouseEvent> = Callback::new(move |_| {
        current_idx.update(|i| *i = i.saturating_sub(1));
    });
    let nav = PlayerNav { on_prev, on_next };

    let handle_ended = move |_| {
        signals.playing.set(false);
        if current_idx.get_untracked() + 1 < items.get_untracked().len() {
            signals.play_after_load.set(true);
            current_idx.update(|i| *i += 1);
        }
    };

    // ── Build handler bundle ──────────────────────────────────────────
    let handlers = PlayerHandlers {
        toggle_play: Callback::new(make_toggle_play(video_ref, signals.playing)),
        toggle_mute: Callback::new(make_toggle_mute(
            video_ref,
            signals.volume,
            signals.last_volume,
            signals.muted,
        )),
        toggle_fullscreen: Callback::new(make_toggle_fullscreen(video_ref)),
        handle_seek: Callback::new(make_handle_seek(video_ref, signals.current_time)),
        handle_volume: Callback::new(make_handle_volume(
            video_ref,
            signals.volume,
            signals.muted,
            signals.last_volume,
        )),
    };

    let handle_loaded_metadata = make_handle_loaded_metadata(video_ref, signals.duration);
    let handle_time_update = make_handle_time_update(video_ref, signals.current_time);

    // ── Install contexts (must precede view!) ─────────────────────────
    PlayerSignals::provide(signals);
    PlayerDerived::provide(derived);
    PlayerHandlers::provide(handlers);
    PlayerNav::provide(nav);

    // ── View ──────────────────────────────────────────────────────────
    let video_class = if audio {
        "lmp-video lmp-video--audio"
    } else {
        "lmp-video"
    };

    let artwork_view = if audio {
        let tc = toggle_controls.clone();
        Some(view! {
            <AudioArtworkOverlay
                artwork=derived.current_artwork
                on_click=move |_| tc()
            />
        })
    } else {
        None
    };

    view! {
        {stylesheet()}
        <div
            class="lmp-player"
            on:mousemove={let show = show_controls.clone(); move |_| show()}
        >
            <video
                node_ref=video_ref
                title=move || derived.current_title.get()
                class=video_class
                on:loadedmetadata=handle_loaded_metadata
                on:timeupdate=handle_time_update
                on:play=move |_| signals.playing.set(true)
                on:pause=move |_| signals.playing.set(false)
                on:ended=handle_ended
                on:click={let tc = toggle_controls.clone(); move |_| tc()}
                playsinline
            />
            {artwork_view}
            <MediaControls
                show_controls=show_controls.clone()
                start_hide_timer=start_hide_timer
                show_fullscreen=!audio
            />
        </div>
    }
}

// ─── Audio artwork overlay ────────────────────────────────────────────────

#[component]
fn AudioArtworkOverlay(
    artwork: Signal<Option<String>>,
    on_click: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <div class="lmp-artwork" on:click=on_click>
            {move || artwork.get().map(|url| view! {
                <div
                    class="lmp-artwork-bg"
                    style=format!("background-image: url('{url}');")
                ></div>
                <img src=url class="lmp-artwork-img" alt=""/>
            })}
        </div>
    }
}

// ─── Controls overlay ─────────────────────────────────────────────────────

#[component]
fn MediaControls(
    show_controls: impl Fn() + Clone + 'static,
    start_hide_timer: impl Fn() + Clone + 'static,
    #[prop(default = true)] show_fullscreen: bool,
) -> impl IntoView {
    let signals = PlayerSignals::expect();
    let derived = PlayerDerived::expect();

    let class = move || {
        if signals.controls_visible.get() {
            "lmp-controls lmp-controls--visible"
        } else {
            "lmp-controls lmp-controls--hidden"
        }
    };
    let on_mouse_leave = {
        let start = start_hide_timer.clone();
        move |_| start()
    };

    view! {
        <div
            class=class
            on:mouseenter={let show = show_controls.clone(); move |_| show()}
            on:mouseleave=on_mouse_leave
            on:touchstart={let show = show_controls.clone(); move |_| show()}
        >
            <div class="lmp-controls-inner">
                <div class="lmp-controls-title">
                    {move || derived.current_title.get()}
                </div>
                <SeekBar/>
                <ControlButtons show_fullscreen=show_fullscreen/>
            </div>
        </div>
    }
}

// ─── Button row ───────────────────────────────────────────────────────────

#[component]
fn ControlButtons(#[prop(default = true)] show_fullscreen: bool) -> impl IntoView {
    let fullscreen_btn = show_fullscreen.then(|| view! { <FullscreenButton/> });

    view! {
        <div class="lmp-controls-row">
            <NavButton direction=NavDirection::Prev/>
            <PlayPauseButton/>
            <NavButton direction=NavDirection::Next/>
            <div class="lmp-volume">
                <MuteButton/>
                <VolumeSlider/>
            </div>
            <div class="lmp-btn--spacer"></div>
            {fullscreen_btn}
        </div>
    }
}

// ─── Individual buttons ───────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum NavDirection {
    Prev,
    Next,
}

impl NavDirection {
    fn aria_label(self) -> &'static str {
        match self {
            Self::Prev => "Previous",
            Self::Next => "Next",
        }
    }
}

#[component]
fn NavButton(direction: NavDirection) -> impl IntoView {
    let derived = PlayerDerived::expect();
    let nav = PlayerNav::expect();

    let show = derived.can_navigate;
    let disabled = Signal::derive(move || match direction {
        NavDirection::Prev => !derived.has_prev.get(),
        NavDirection::Next => !derived.has_next.get(),
    });
    let on_click = match direction {
        NavDirection::Prev => nav.on_prev,
        NavDirection::Next => nav.on_next,
    };
    let label = direction.aria_label();

    view! {
        <Show when=move || show.get()>
            <button
                class="lmp-btn"
                on:click=move |ev| on_click.run(ev)
                disabled=move || disabled.get()
                aria-label=label
            >
                {move || match direction {
                    NavDirection::Prev => Either::Left(view! { <PrevPageIcon/> }),
                    NavDirection::Next => Either::Right(view! { <NextPageIcon/> }),
                }}
            </button>
        </Show>
    }
}

#[component]
fn PlayPauseButton() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <button
            class="lmp-btn"
            on:click=move |ev| handlers.toggle_play.run(ev)
            aria-label="Play or pause"
        >
            {move || if signals.playing.get() {
                Either::Left(PauseIcon())
            } else {
                Either::Right(PlayIcon())
            }}
        </button>
    }
}

#[component]
fn MuteButton() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <button
            class="lmp-btn"
            on:click=move |ev| handlers.toggle_mute.run(ev)
            aria-label="Mute or unmute"
        >
            {move || if signals.muted.get() || signals.volume.get() == 0.0 {
                Either::Left(MuteIcon())
            } else {
                Either::Right(VolumeIcon())
            }}
        </button>
    }
}

#[component]
fn FullscreenButton() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <button
            class="lmp-btn"
            on:click=move |ev| handlers.toggle_fullscreen.run(ev)
            aria-label="Toggle fullscreen"
        >
            {move || if signals.fullscreen.get() {
                Either::Left(FullscreenExitIcon())
            } else {
                Either::Right(FullscreenIcon())
            }}
        </button>
    }
}

#[component]
fn VolumeSlider() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    let vol_value = move || {
        if signals.muted.get() {
            0.0
        } else {
            signals.volume.get()
        }
    };

    view! {
        <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            class="lmp-slider lmp-slider--volume"
            prop:value=vol_value
            on:input=move |ev| handlers.handle_volume.run(ev)
        />
    }
}

// ─── Seek bar ─────────────────────────────────────────────────────────────

#[component]
fn SeekBar() -> impl IntoView {
    let signals = PlayerSignals::expect();
    let handlers = PlayerHandlers::expect();

    view! {
        <div class="lmp-seek">
            <span class="lmp-time">
                {move || format_time(signals.current_time.get())}
            </span>
            <input
                type="range"
                min="0"
                class="lmp-slider lmp-slider--seek"
                prop:max=signals.duration
                prop:value=signals.current_time
                on:input=move |ev| handlers.handle_seek.run(ev)
            />
            <span class="lmp-time">
                {move || format_time(signals.duration.get())}
            </span>
        </div>
    }
}

// ─── Util ─────────────────────────────────────────────────────────────────

fn format_time(time: f64) -> String {
    if time.is_nan() {
        return "00:00".into();
    }
    let t = time as u64;
    let h = t / 3600;
    let m = (t % 3600) / 60;
    let s = t % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}
