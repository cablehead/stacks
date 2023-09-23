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
    pub previews: Vec<String>,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct Nav {
    pub root: Option<Layer>,
    pub sub: Option<Layer>,
    pub undo: Option<Item>,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct UI {
    pub focused: Option<view::Focus>,
    pub last_selected: HashMap<Scru128Id, view::Focus>,
    pub matches: Option<HashSet<ssri::Integrity>>,
    pub view: view::View,
    pub theme_mode: String,
}

impl UI {
    pub fn new(v: &view::View) -> Self {
        Self {
            focused: None,
            last_selected: HashMap::new(),
            matches: None,
            view: v.clone(),
            theme_mode: "".to_string(),
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

    pub fn select(&mut self, focus: Option<view::Focus>) {
        if let Some(focus) = focus.as_ref() {
            if let Some(stack_id) = focus.item.stack_id {
                self.last_selected.insert(stack_id, focus.clone());
            }
        }
        self.focused = focus;
    }

    pub fn select_up(&mut self) {
        self.select(self.view.get_best_focus_prev(&self.focused));
    }

    pub fn select_down(&mut self) {
        let focused = self.focused.clone().or(self.view.first());
        self.select(self.view.get_best_focus_next(&focused));
    }

    pub fn select_left(&mut self) {
        let focused = { self.view.get_best_focus(&self.focused) };
        let target = focused
            .and_then(|focused| focused.item.stack_id)
            .and_then(|id| self.view.get_focus_for_id(&id));
        if let Some(target) = target {
            self.select(Some(target));
        }
    }

    pub fn select_right(&mut self) {
        let target = self.view.get_best_focus(&self.focused).and_then(|focused| {
            self.last_selected
                .get(&focused.item.id)
                .cloned()
                .or_else(|| {
                    self.view
                        .children(&focused.item)
                        .first()
                        .and_then(|id| self.view.get_focus_for_id(id))
                })
        });
        self.select(target);
    }

    pub fn render(&self, store: &Store) -> Nav {
        let focused = self.view.get_best_focus(&self.focused);
        if focused.is_none() {
            return Nav {
                root: None,
                sub: None,
                undo: self.view.undo.as_ref().map(|item| with_meta(store, item)),
            };
        }
        let focused = focused.unwrap();

        // the sub layer is focused
        if let Some(stack_id) = focused.item.stack_id {
            let items: Vec<_> = self
                .view
                .get_peers(&focused.item)
                .iter()
                .cloned()
                .map(|item| with_meta(store, item))
                .collect();
            let selected = with_meta(store, &focused.item);

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
                    previews: Vec::new(),
                }),
                sub: Some(Layer {
                    items: items.clone(),
                    selected,
                    is_focus: true,
                    previews: items
                        .iter()
                        .map(|item| {
                            let content = store.get_content(&item.hash);
                            generate_preview(&self.theme_mode, &item, &content)
                        })
                        .collect(),
                }),
                undo: self.view.undo.as_ref().map(|item| with_meta(store, item)),
            }
        } else {
            // the root layer is focused
            let children: Vec<_> = self
                .view
                .children(&focused.item)
                .iter()
                .map(|id| self.view.items.get(id).unwrap().clone())
                .collect();

            let sub = if !children.is_empty() {
                let possible = self.last_selected.get(&focused.item.id).cloned();
                let possible =
                    possible.or(self.view.get_focus_for_id(&children.get(0).unwrap().id));
                let selected = self.view.get_best_focus(&possible).unwrap();
                let selected = with_meta(store, &selected.item);
                let items: Vec<_> = children.iter().map(|item| with_meta(store, item)).collect();

                Some(Layer {
                    items: items.clone(),
                    selected,
                    is_focus: false,
                    previews: items
                        .iter()
                        .map(|item| {
                            let content = store.get_content(&item.hash);
                            generate_preview(&self.theme_mode, &item, &content)
                        })
                        .collect(),
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
                    selected: with_meta(store, &focused.item),
                    is_focus: true,
                    previews: Vec::new(),
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

pub fn markdown_to_html(theme_mode: &str, input: &Vec<u8>) -> String {
    let adapter = SyntectAdapter::new(&format!("base16-ocean.{}", theme_mode));
    let options = ComrakOptions::default();
    let mut plugins = ComrakPlugins::default();

    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let input_str = String::from_utf8(input.clone()).unwrap();
    markdown_to_html_with_plugins(&input_str, &options, &plugins)
}

use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

pub fn code_to_html(theme_mode: &str, input: &Vec<u8>, ext: &str) -> String {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension(ext).unwrap();
    let theme = &ts.themes[&format!("base16-ocean.{}", theme_mode)];
    let input_str = String::from_utf8(input.clone()).unwrap();
    let highlighted_html = highlighted_html_for_string(&input_str, &ps, syntax, theme);
    highlighted_html.unwrap()
}

use maud::html;

fn generate_preview(theme_mode: &str, item: &Item, content: &Option<Vec<u8>>) -> String {
    let file_extensions: HashMap<&str, &str> = [
        ("Rust", "rs"),
        ("JSON", "json"),
        ("Python", "py"),
        ("JavaScript", "js"),
        ("HTML", "html"),
        ("Shell", "sh"),
        ("Go", "go"),
        ("Ruby", "rb"),
        ("SQL", "sql"),
        ("XML", "xml"),
        ("YAML", "yaml"),
    ]
    .iter()
    .cloned()
    .collect();

    match content {
        None => "loading...".to_string(),
        Some(data) => {
            if item.mime_type == MimeType::ImagePng {
                let img_data = format!("data:image/png;base64,{}", util::b64encode(data));
                let img = html! {
                    img src=(img_data) style="opacity: 0.95; border-radius: 0.5rem; max-height: 100%; height: auto; width: auto; object-fit: contain";
                };
                img.into_string()
            } else if item.content_type == "Markdown" {
                let md_html = markdown_to_html(theme_mode, data);
                let md_html = maud::PreEscaped(md_html);
                let div = html! {
                    div.("scroll-me")[item.ephemeral] .preview.markdown {
                        (md_html)
                    }
                };
                div.into_string()
            } else if let Some(ext) = file_extensions.get(item.content_type.as_str()) {
                let html = code_to_html(theme_mode, data, ext);
                let html = maud::PreEscaped(html);
                let div = html! {
                    div.("scroll-me")[item.ephemeral] .preview.rust {
                        (html)
                    }
                };
                div.into_string()
            } else {
                let data = String::from_utf8(data.clone()).unwrap();
                let pre = html! {
                    pre.("scroll-me")[item.ephemeral] style="margin: 0; white-space: pre-wrap; overflow-x: hidden" {
                        (data)
                    }
                };
                pre.into_string()
            }
        }
    }
}
