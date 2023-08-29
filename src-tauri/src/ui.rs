use std::collections::HashMap;

use scru128::Scru128Id;
use ssri::Integrity;

pub use crate::store::{MimeType, Packet, Store};

use crate::view;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Layer {
    pub items: Vec<Item>,
    pub selected: Item,
    pub is_focus: bool,
}

#[derive(Debug, Clone)]
pub struct Nav {
    pub root: Layer,
    pub sub: Option<Layer>,
}

#[derive(Debug, Clone)]
pub struct UI {
    pub focused_id: Scru128Id,
    pub last_selected: HashMap<Scru128Id, Scru128Id>,
    pub filter: String,
}

impl UI {
    pub fn render(&self, store: &Store, v: &view::View) -> Nav {
        let _matches = if !self.filter.is_empty() {
            Some(store.index.query(&self.filter))
        } else {
            None
        };

        let id_to_item = |item: &view::Item| -> Item {
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
            }
        };

        let focused = v.items.get(&self.focused_id).unwrap().clone();
        let root_id = focused.stack_id.unwrap_or(focused.id);

        let root_items = v.root();
        let root_selected = v.items.get(&root_id).unwrap();

        let sub_items = v
            .children(root_selected)
            .iter()
            .map(|id| v.items.get(id).unwrap().clone())
            .collect::<Vec<_>>();
        let sub_selected = sub_items[0].clone();

        let root_items = root_items.iter().map(id_to_item).collect::<Vec<_>>();
        let root_selected = id_to_item(root_selected);
        let root_is_focus = self.focused_id == root_selected.id;

        let sub_items = sub_items.iter().map(id_to_item).collect::<Vec<_>>();
        let sub_selected = id_to_item(&sub_selected);
        let sub_is_focus = self.focused_id == sub_selected.id;

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
