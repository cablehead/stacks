use std::collections::{HashMap, HashSet};

use scru128::Scru128Id;
use ssri::Integrity;

pub use crate::store::{MimeType, Packet, Store};

use crate::view;

#[derive(serde::Serialize, Debug, Clone)]
pub struct Item {
    pub id: Scru128Id,
    pub stack_id: Option<Scru128Id>,
    pub last_touched: Scru128Id,
    pub touched: Vec<Scru128Id>,
    pub hash: Option<Integrity>,
    pub mime_type: MimeType,
    pub content_type: String,
    pub terse: String,
    pub tiktokens: usize,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct Layer {
    pub items: Vec<Item>,
    pub selected: Item,
    pub is_focus: bool,
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
                }),
                sub: Some(Layer {
                    items: self
                        .view
                        .get_peers(focused)
                        .iter()
                        .cloned()
                        .map(|item| with_meta(store, item))
                        .collect(),
                    selected: with_meta(store, focused),
                    is_focus: true,
                }),
                undo: self.view.undo.as_ref().map(|item| with_meta(store, item)),
            }
        } else {
            let children: Vec<_> = self
                .view
                .children(focused)
                .iter()
                .map(|id| self.view.items.get(id).unwrap().clone())
                .collect();

            let sub = if !children.is_empty() {
                let possible = self.last_selected.get(&focused.id).or(children.get(0));
                let selected = self.view.get_best_focus(possible).unwrap();

                Some(Layer {
                    items: children.iter().map(|item| with_meta(store, item)).collect(),
                    selected: with_meta(store, selected),
                    is_focus: false,
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
                }),
                sub,
                undo: self.view.undo.as_ref().map(|item| with_meta(store, item)),
            }
        }
    }
}

pub fn with_meta(store: &Store, item: &view::Item) -> Item {
    if let Some(hash) = &item.hash {
        let content_meta = store.content_meta_cache.get(&hash).unwrap();
        Item {
            id: item.id,
            stack_id: item.stack_id,
            last_touched: item.last_touched,
            touched: item.touched.clone(),
            hash: Some(hash.clone()),
            mime_type: content_meta.mime_type.clone(),
            content_type: content_meta.content_type.clone(),
            terse: content_meta.terse.clone(),
            tiktokens: content_meta.tiktokens,
        }
    } else {
        Item {
            id: item.id,
            stack_id: item.stack_id,
            last_touched: item.last_touched,
            touched: item.touched.clone(),
            hash: None,
            mime_type: MimeType::TextPlain,
            content_type: "Text".to_string(),
            terse: "".to_string(),
            tiktokens: 0,
        }
    }
}
