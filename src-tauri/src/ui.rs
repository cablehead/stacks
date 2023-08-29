use std::collections::HashMap;

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
    pub hash: Integrity,
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
    pub root: Layer,
    pub sub: Option<Layer>,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct UI {
    pub focused_id: Option<Scru128Id>,
    pub last_selected: HashMap<Scru128Id, Scru128Id>,
    pub filter: String,
}

impl UI {
    pub fn new(v: &view::View) -> Self {
        let focused_id = {
            let root = &v.root()[0];
            if !root.children.is_empty() {
                Some(v.children(root)[0])
            } else {
                Some(root.id)
            }
        };

        Self {
            focused_id,
            last_selected: HashMap::new(),
            filter: String::new(),
        }
    }

    pub fn select(&mut self, v: &view::View, id: Scru128Id) {
        if let Some(item) = v.items.get(&id) {
            if let Some(stack_id) = item.stack_id {
                self.last_selected.insert(stack_id, id);
            }
            self.focused_id = Some(id);
        }
    }

    pub fn select_up(&mut self, v: &view::View) {
        if let Some(focused_id) = self.focused_id {
            let peers = v.get_peers(&focused_id);
            let current_index = peers.iter().position(|id| id == &focused_id);
            if let Some(index) = current_index {
                if index > 0 {
                    self.focused_id = Some(peers[index - 1]);
                    if let Some(item) = v.items.get(&peers[index - 1]) {
                        if let Some(stack_id) = item.stack_id {
                            self.last_selected.insert(stack_id, peers[index - 1]);
                        }
                    }
                }
            }
        }
    }

    pub fn select_down(&mut self, v: &view::View) {
        if let Some(focused_id) = self.focused_id {
            let peers = v.get_peers(&focused_id);
            let current_index = peers.iter().position(|id| id == &focused_id);
            if let Some(index) = current_index {
                if index < peers.len() - 1 {
                    self.focused_id = Some(peers[index + 1]);
                    if let Some(item) = v.items.get(&peers[index + 1]) {
                        if let Some(stack_id) = item.stack_id {
                            self.last_selected.insert(stack_id, peers[index + 1]);
                        }
                    }
                }
            }
        }
    }

    pub fn select_left(&mut self, v: &view::View) {
        if let Some(focused_id) = self.focused_id {
            if let Some(item) = v.items.get(&focused_id) {
                if let Some(stack_id) = item.stack_id {
                    self.focused_id = Some(stack_id);
                }
            }
        }
    }

    pub fn select_right(&mut self, v: &view::View) {
        if let Some(focused_id) = self.focused_id {
            if let Some(item) = v.items.get(&focused_id) {
                let children = v.children(item);
                if !children.is_empty() {
                    let next_id = self
                        .last_selected
                        .get(&focused_id)
                        .cloned()
                        .unwrap_or(children[0]);
                    self.focused_id = Some(next_id);
                }
            }
        }
    }

    pub fn render(&self, store: &Store, v: &view::View) -> Nav {
        let _matches = if !self.filter.is_empty() {
            Some(store.index.query(&self.filter))
        } else {
            None
        };

        let id_to_item = |item: &view::Item| -> Item {
            let content_meta = store.content_meta_cache.get(&item.hash).unwrap();
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
            }
        };

        let focused_id = self.focused_id.unwrap_or_else(|| {
            let root = &v.root()[0];
            if !root.children.is_empty() {
                v.children(root)[0]
            } else {
                root.id
            }
        });

        let focused = v.items.get(&focused_id).unwrap().clone();
        let root_id = focused.stack_id.unwrap_or(focused.id);

        let root_items = v.root();
        let root_selected = v.items.get(&root_id).unwrap();

        let sub_items = v
            .children(root_selected)
            .iter()
            .map(|id| v.items.get(id).unwrap().clone())
            .collect::<Vec<_>>();
        let sub_selected = sub_items
            .iter()
            .find(|item| item.id == focused_id)
            .unwrap_or(&sub_items[0])
            .clone();

        let root_items = root_items.iter().map(id_to_item).collect::<Vec<_>>();
        let root_selected = id_to_item(root_selected);
        let root_is_focus = focused_id == root_selected.id;

        let sub_items = sub_items.iter().map(id_to_item).collect::<Vec<_>>();
        let sub_selected = id_to_item(&sub_selected);
        let sub_is_focus = focused_id == sub_selected.id;

        Nav {
            root: Layer {
                items: root_items,
                selected: root_selected,
                is_focus: root_is_focus,
            },
            sub: Some(Layer {
                items: sub_items,
                selected: sub_selected,
                is_focus: sub_is_focus,
            }),
        }
    }
}
