//! A code editor searched through the application's own search bar.
//!
//! The editor keeps its matching, highlighting, scrolling and replacing; only
//! the UI on top is the application's. Two things make that possible:
//!
//! - `searchable(false)` keeps the built-in panel closed and lets `Ctrl-F` /
//!   `Cmd-F` bubble up to this view, which focuses its own search field.
//! - `set_search_query` runs a search without the panel. The editor
//!   highlights the matches, and `next_search_match`,
//!   `previous_search_match` and `search_session()` drive and describe them.

use gpui_kit::assets::Assets;
use gpui_kit::component::{
    ActiveTheme, Disableable, Icon, IconName, Root, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Editor, EditorState, Input, InputEvent, InputState, Search, TabSize},
    v_flex,
};
use gpui_kit::*;

pub struct Example {
    editor: Entity<EditorState>,
    search: Entity<InputState>,

    /// Kept alive with the view, so the search field stops driving the editor
    /// when the view goes away.
    _subscriptions: Vec<Subscription>,
}

impl Example {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("rust")
                .line_number(true)
                .indent_guides(true)
                .tab_size(TabSize {
                    tab_size: 4,
                    hard_tabs: false,
                })
                .soft_wrap(false)
                // Leave the search shortcut to this view instead of the
                // built-in panel.
                .searchable(false)
                .default_value(include_str!("../../editor/fixtures/test.rs"))
        });
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search"));

        // Every keystroke in the search field becomes the editor's query.
        let _subscriptions = vec![cx.subscribe(
            &search,
            |this: &mut Self, search, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    let query = search.read(cx).value();
                    this.editor
                        .update(cx, |editor, cx| editor.set_search_query(query, true, cx));
                    cx.notify();
                }
            },
        )];

        Self {
            editor,
            search,
            _subscriptions,
        }
    }

    /// `2/5`: the current match and the total, as the built-in panel shows it.
    fn match_label(&self, cx: &App) -> String {
        self.editor.read(cx).search_session().matcher.label()
    }

    fn has_matches(&self, cx: &App) -> bool {
        !self.editor.read(cx).search_session().matcher.is_empty()
    }

    fn previous_match(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| {
            editor.previous_search_match(cx);
        });
        cx.notify();
    }

    fn next_match(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| {
            editor.next_search_match(cx);
        });
        cx.notify();
    }

    /// Ends the search: clears the field and the editor's highlights.
    fn close_search(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.search
            .update(cx, |search, cx| search.set_value("", window, cx));
        self.editor.update(cx, |editor, cx| editor.close_search(cx));
        cx.notify();
    }

    /// The editor is not `searchable`, so `Ctrl-F` / `Cmd-F` pressed inside
    /// it reaches this view.
    fn on_action_search(&mut self, _: &Search, window: &mut Window, cx: &mut Context<Self>) {
        self.search.update(cx, |search, cx| {
            search.focus(window, cx);
            search.select_all(window, cx);
        });
    }
}

impl Render for Example {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_matches = self.has_matches(cx);

        v_flex()
            .size_full()
            .p_4()
            .gap_2()
            .on_action(cx.listener(Self::on_action_search))
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Input::new(&self.search)
                            .small()
                            .w_64()
                            .prefix(Icon::new(IconName::Search).small()),
                    )
                    .child(
                        Button::new("previous-match")
                            .xsmall()
                            .ghost()
                            .icon(IconName::ChevronLeft)
                            .tooltip("Previous match")
                            .disabled(!has_matches)
                            .on_click(cx.listener(Self::previous_match)),
                    )
                    .child(
                        Button::new("next-match")
                            .xsmall()
                            .ghost()
                            .icon(IconName::ChevronRight)
                            .tooltip("Next match")
                            .disabled(!has_matches)
                            .on_click(cx.listener(Self::next_match)),
                    )
                    .child(
                        div()
                            .min_w_16()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(self.match_label(cx)),
                    )
                    .child(
                        Button::new("close-search")
                            .xsmall()
                            .ghost()
                            .icon(IconName::Close)
                            .tooltip("Close search")
                            .on_click(cx.listener(Self::close_search)),
                    ),
            )
            .child(
                Editor::new(&self.editor)
                    .flex_1()
                    .w_full()
                    .font_family(cx.theme().mono_font_family.clone())
                    .text_size(cx.theme().mono_font_size),
            )
    }
}

fn main() {
    let app = gpui_kit::application().with_assets(Assets);

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_kit::init(cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(960.), px(640.)), cx)),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| Example::new(window, cx));
                // The first level view in a window must be a Root.
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
