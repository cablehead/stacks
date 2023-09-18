use std::collections::{HashMap, HashSet};

use scru128::Scru128Id;
use ssri::Integrity;

pub use crate::store::{MimeType, Packet, Store};

use crate::util;
use crate::view;

#[derive(serde::Serialize, Debug, Clone)]
pub struct Item {
    pub id: Scru128Id,
    pub stack_id: Option<Scru128Id>,
    pub last_touched: Scru128Id,
    pub touched: Vec<Scru128Id>,
    pub hash: Integrity,
    pub mime_type: MimeType,
    pub content_type: String,
    pub terse: String,
    pub tiktokens: usize,
    pub ephemeral: bool,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct Layer {
    pub items: Vec<Item>,
    pub selected: Item,
    pub is_focus: bool,
    pub preview: String,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct Nav {
    pub root: Option<Layer>,
    pub sub: Option<Layer>,
    pub undo: Option<Item>,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct UI {
    pub focused: Option<view::Item>,
    pub last_selected: HashMap<Scru128Id, view::Item>,
    pub matches: Option<HashSet<ssri::Integrity>>,
    pub view: view::View,
}

impl UI {
    pub fn new(v: &view::View) -> Self {
        Self {
            focused: None,
            last_selected: HashMap::new(),
            matches: None,
            view: v.clone(),
        }
    }

    pub fn reset(&mut self, v: view::View) {
        self.focused = None;
        self.last_selected = HashMap::new();
        self.matches = None;
        self.view = v;
    }

    pub fn set_filter(&mut self, store: &Store, v: &view::View, filter: &str, content_type: &str) {
        self.matches = if !filter.is_empty() || (content_type != "All" && !content_type.is_empty())
        {
            let matches = store.query(filter, content_type);
            Some(matches)
        } else {
            None
        };
        self.refresh_view(v);
    }

    pub fn refresh_view(&mut self, v: &view::View) {
        self.view = if let Some(matches) = &self.matches {
            v.filter(matches)
        } else {
            v.clone()
        }
    }

    pub fn select(&mut self, item: Option<&view::Item>) {
        self.focused = item.cloned();
        if let Some(item) = item {
            if let Some(stack_id) = item.stack_id {
                self.last_selected.insert(stack_id, item.clone());
            }
        }
    }

    pub fn select_up(&mut self) {
        if let Some(focused) = self.view.get_best_focus(self.focused.as_ref()) {
            let view = self.view.clone();
            let peers = view.get_peers(focused);
            let index = peers
                .iter()
                .cloned()
                .position(|peer| peer.last_touched <= focused.last_touched)
                .unwrap();
            if index > 0 {
                self.select(peers.get(index - 1).cloned());
            }
        }
    }

    pub fn select_down(&mut self) {
        if let Some(focused) = self.view.get_best_focus(self.focused.as_ref()) {
            let view = self.view.clone();
            let peers = view.get_peers(focused);
            let index = peers
                .iter()
                .position(|peer| peer.last_touched <= focused.last_touched)
                .unwrap();
            if index < peers.len() - 1 {
                self.select(peers.get(index + 1).cloned());
            }
        }
    }

    pub fn select_left(&mut self) {
        let focused = { self.view.get_best_focus(self.focused.as_ref()).cloned() };
        let target = {
            focused
                .and_then(|focused| focused.stack_id)
                .and_then(|id| self.view.items.get(&id))
                .cloned()
        };
        if let Some(target) = target {
            self.select(Some(&target));
        }
    }

    pub fn select_right(&mut self) {
        let target = self
            .view
            .get_best_focus(self.focused.as_ref())
            .and_then(|focused| {
                self.last_selected
                    .get(&focused.id)
                    .or_else(|| {
                        self.view
                            .children(focused)
                            .first()
                            .and_then(|id| self.view.items.get(id))
                    })
                    .cloned()
            });
        self.select(target.as_ref());
    }

    pub fn render(&self, store: &Store) -> Nav {
        let focused = self.view.get_best_focus(self.focused.as_ref());
        if focused.is_none() {
            return Nav {
                root: None,
                sub: None,
                undo: self.view.undo.as_ref().map(|item| with_meta(store, item)),
            };
        }
        let focused = focused.unwrap();

        // the sub layer is focused
        if let Some(stack_id) = focused.stack_id {
            let selected = with_meta(store, focused);
            let content = store.get_content(&selected.hash);
            let preview = generate_preview(&selected, &content);

            Nav {
                root: Some(Layer {
                    items: self
                        .view
                        .root()
                        .iter()
                        .map(|item| with_meta(store, item))
                        .collect(),
                    selected: with_meta(store, self.view.items.get(&stack_id).unwrap()),
                    is_focus: false,
                    preview: "".to_string(),
                }),
                sub: Some(Layer {
                    items: self
                        .view
                        .get_peers(focused)
                        .iter()
                        .cloned()
                        .map(|item| with_meta(store, item))
                        .collect(),
                    selected,
                    is_focus: true,
                    preview: preview,
                }),
                undo: self.view.undo.as_ref().map(|item| with_meta(store, item)),
            }
        } else {
            // the root layer is focused
            let children: Vec<_> = self
                .view
                .children(focused)
                .iter()
                .map(|id| self.view.items.get(id).unwrap().clone())
                .collect();

            let sub = if !children.is_empty() {
                let possible = self.last_selected.get(&focused.id).or(children.get(0));
                let selected = self.view.get_best_focus(possible).unwrap();
                let selected = with_meta(store, selected);
                let content = store.get_content(&selected.hash);
                let preview = generate_preview(&selected, &content);

                Some(Layer {
                    items: children.iter().map(|item| with_meta(store, item)).collect(),
                    selected,
                    is_focus: false,
                    preview,
                })
            } else {
                None
            };

            Nav {
                root: Some(Layer {
                    items: self
                        .view
                        .root()
                        .iter()
                        .cloned()
                        .map(|item| with_meta(store, item))
                        .collect(),
                    selected: with_meta(store, focused),
                    is_focus: true,
                    preview: "".to_string(),
                }),
                sub,
                undo: self.view.undo.as_ref().map(|item| with_meta(store, item)),
            }
        }
    }
}

pub fn with_meta(store: &Store, item: &view::Item) -> Item {
    let content_meta = store.get_content_meta(&item.hash).unwrap();
    Item {
        id: item.id,
        stack_id: item.stack_id,
        last_touched: item.last_touched,
        touched: item.touched.clone(),
        hash: item.hash.clone(),
        mime_type: content_meta.mime_type.clone(),
        content_type: content_meta.content_type.clone(),
        terse: content_meta.terse.clone(),
        tiktokens: content_meta.tiktokens,
        ephemeral: item.ephemeral,
    }
}

use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, ComrakOptions, ComrakPlugins};

pub fn markdown_to_html(input: &Vec<u8>) -> String {
    let adapter = SyntectAdapter::new("base16-ocean.dark");
    let options = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let input_str = String::from_utf8(input.clone()).unwrap();
    markdown_to_html_with_plugins(&input_str, &options, &plugins)
}

fn generate_preview(item: &Item, content: &Option<Vec<u8>>) -> String {
    match content {
        None => "loading...".to_string(),
        Some(data) => {
            if item.mime_type == MimeType::ImagePng {
                format!("<img src=\"data:image/png;base64,{}\" style=\"opacity: 0.95; border-radius: 0.5rem; max-height: 100%; height: auto; width: auto; object-fit: contain\" />", util::b64encode(data))
            } else {
                if true { // item.id.to_string() == "03AA4778N243DNF96I8NDK7DP" {
                    format!(
                        "<div class=\"scroll-me\" style=\"margin: 0\">{}</div>",
                        markdown_to_html(data)
                    )
                } else {
                    format!(
    "<pre class=\"scroll-me\" style=\"margin: 0; white-space: pre-wrap; overflow-x: hidden\">{}</pre>",
    std::str::from_utf8(&data).unwrap()
)
                }
            }
        }
    }
}
