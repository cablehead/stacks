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
    pub focused: Option<view::Item>,
    pub last_selected: HashMap<Scru128Id, view::Item>,
    pub filter: String,
    pub matches: Option<HashSet<ssri::Integrity>>,
}

impl UI {
    pub fn new(v: &view::View) -> Self {
        Self {
            focused: None,
            last_selected: HashMap::new(),
            filter: String::new(),
            matches: None,
        }
    }

    pub fn set_filter(&mut self, store: &Store, filter: &str, content_type: &str) {
        self.filter = filter.into();
        self.matches = Some(store.query(filter, content_type));
    }

    pub fn select(&mut self, v: &view::View, id: Scru128Id) {
        if let Some(item) = v.items.get(&id) {
            if let Some(stack_id) = item.stack_id {
                self.last_selected.insert(stack_id, item.clone());
            }
            self.focused = Some(item.clone());
        }
    }

    pub fn select_up(&mut self, v: &view::View) {
        if let Some(focused) = self.focused.as_ref().or(v.first().as_ref()) {
            let peers = v.get_peers(&focused);
            let current_index = peers.iter().position(|peer| peer.id == focused.id);
            if let Some(index) = current_index {
                if index > 0 {
                    self.select(v, peers[index - 1].id);
                }
            }
        }
    }

    pub fn select_down(&mut self, v: &view::View) {
        if let Some(focused) = self.focused.as_ref().or(v.first().as_ref()) {
            let peers = v.get_peers(&focused);
            let current_index = peers.iter().position(|peer| peer.id == focused.id);
            if let Some(index) = current_index {
                if index < peers.len() - 1 {
                    self.select(v, peers[index + 1].id);
                }
            }
        }
    }

    pub fn select_left(&mut self, v: &view::View) {
        if let Some(focused) = self.focused.as_ref().or(v.first().as_ref()) {
            if let Some(stack_id) = focused.stack_id {
                self.select(v, stack_id);
            }
        }
    }

    pub fn select_right(&mut self, v: &view::View) {
        if let Some(focused) = self.focused.as_ref().or(v.first().as_ref()) {
            let children = v.children(&focused);
            if children.is_empty() {
                return;
            }

            if let Some(child) = self
                .last_selected
                .get(&focused.id)
                .or(v.items.get(&children[0]))
            {
                self.select(v, child.id);
            }
        }
    }

    pub fn render(&self, store: &Store, v: &view::View) -> Nav {
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

        let root = v.root();
        let root = root
            .iter()
            .filter_map(|stack| {
                let children = v.children(stack);
                let filtered_children = children
                    .iter()
                    .filter_map(|id| {
                        let item = v.items.get(id).unwrap().clone();
                        if let Some(matches) = &self.matches {
                            if matches.contains(&item.hash) {
                                Some(item)
                            } else {
                                None
                            }
                        } else {
                            Some(item)
                        }
                    })
                    .collect::<Vec<_>>();
                if !filtered_children.is_empty() || self.matches.is_none() {
                    Some((stack.clone(), filtered_children))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // println!("{:?}", root);

        let focused = self.focused.clone().or(v.first().clone()).unwrap();

        let root_id = focused.stack_id.unwrap_or(focused.id);

        let root_items = v.root();
        let root_selected = v.items.get(&root_id).unwrap();

        let sub_items = v
            .children(root_selected)
            .iter()
            .map(|id| v.items.get(id).unwrap().clone())
            .collect::<Vec<_>>();
        let sub_selected = self
            .last_selected
            .get(&root_selected.id)
            .cloned()
            .unwrap_or(sub_items[0].clone());

        let root_items = root_items.iter().map(id_to_item).collect::<Vec<_>>();
        let root_selected = id_to_item(root_selected);
        let root_is_focus = focused.id == root_selected.id;

        let sub_items = sub_items.iter().map(id_to_item).collect::<Vec<_>>();
        let sub_selected = id_to_item(&sub_selected);
        let sub_is_focus = focused.id == sub_selected.id;

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
