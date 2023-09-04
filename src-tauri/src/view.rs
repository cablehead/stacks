use std::collections::{HashMap, HashSet};

use scru128::Scru128Id;
use ssri::Integrity;

use crate::store::Packet;

#[derive(serde::Serialize, Debug, Clone)]
pub struct Item {
    pub id: Scru128Id,
    pub last_touched: Scru128Id,
    pub touched: Vec<Scru128Id>,
    pub hash: Integrity,
    pub stack_id: Option<Scru128Id>,
    pub children: Vec<Scru128Id>,
    pub forked_children: Vec<Scru128Id>,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct View {
    pub items: HashMap<Scru128Id, Item>,
    pub undo: Option<Item>,
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}

impl View {
    pub fn new() -> Self {
        View {
            items: HashMap::new(),
            undo: None,
        }
    }

    pub fn merge(&mut self, packet: Packet) {
        match packet {
            Packet::Add(packet) => {
                // Check if an item with the same hash already exists in the same stack
                if let Some(stack_id) = packet.stack_id {
                    if let Some(stack) = self.items.get(&stack_id) {
                        let children = stack.children.clone();
                        for child_id in children {
                            if let Some(child) = self.items.get_mut(&child_id) {
                                if child.hash == packet.hash {
                                    // If it exists, update it
                                    child.touched.push(packet.id);
                                    child.last_touched = packet.id;
                                    if let Some(stack) =
                                        child.stack_id.and_then(|id| self.items.get_mut(&id))
                                    {
                                        stack.last_touched = packet.id;
                                    }
                                    return;
                                }
                            }
                        }
                    }
                }

                // If it doesn't exist, add it
                let item = Item {
                    id: packet.id,
                    last_touched: packet.id,
                    touched: vec![packet.id],
                    hash: packet.hash,
                    stack_id: packet.stack_id,
                    children: Vec::new(),
                    forked_children: Vec::new(),
                };

                if let Some(stack) = packet.stack_id.and_then(|id| self.items.get_mut(&id)) {
                    stack.children.push(packet.id);
                    stack.last_touched = packet.id;
                }
                self.items.insert(packet.id, item);
            }

            Packet::Update(packet) => {
                if let Some(item) = self.items.get(&packet.source_id).cloned() {
                    let mut item = item;

                    if let Some(hash) = packet.hash {
                        item.hash = hash;
                    }

                    if let Some(new_stack_id) = packet.stack_id {
                        if let Some(old_stack) =
                            item.stack_id.and_then(|id| self.items.get_mut(&id))
                        {
                            old_stack.children.retain(|&id| id != packet.source_id);
                        }
                        item.stack_id = Some(new_stack_id);
                        if let Some(new_stack) = self.items.get_mut(&new_stack_id) {
                            new_stack.children.push(packet.source_id);
                        }
                    }

                    item.touched.push(packet.id);
                    item.last_touched = packet.id;
                    if let Some(stack) = item.stack_id.and_then(|id| self.items.get_mut(&id)) {
                        stack.last_touched = packet.id;
                    }

                    self.items.insert(packet.source_id, item);
                }
            }

            Packet::Fork(packet) => {
                if let Some(item) = self.items.get(&packet.source_id) {
                    let mut new_item = item.clone();
                    new_item.id = packet.id;

                    new_item.forked_children = item.children.clone();
                    new_item.children = Vec::new();

                    if let Some(hash) = packet.hash {
                        new_item.hash = hash;
                    }

                    if let Some(new_stack_id) = packet.stack_id {
                        new_item.stack_id = Some(new_stack_id);
                    }

                    new_item.touched.push(packet.id);
                    new_item.last_touched = packet.id;

                    if let Some(stack) = new_item.stack_id.and_then(|id| self.items.get_mut(&id)) {
                        // Remove the forked item from forked_children
                        stack.forked_children.retain(|&id| id != packet.source_id);
                        // And add the new item to children
                        stack.children.push(packet.id);
                        stack.last_touched = packet.id;
                    }

                    self.items.insert(packet.id, new_item);
                }
            }

            Packet::Delete(packet) => {
                if let Some(mut item) = self.items.remove(&packet.source_id) {
                    if let Some(stack) = item.stack_id.and_then(|id| self.items.get_mut(&id)) {
                        stack.children.retain(|&id| id != packet.source_id);
                        stack.last_touched = packet.id;
                    }
                    item.last_touched = packet.id;
                    self.undo = Some(item);
                }
            }
        }
    }

    pub fn root(&self) -> Vec<&Item> {
        let mut root_items = self
            .items
            .values()
            .filter(|item| item.stack_id.is_none())
            .collect::<Vec<_>>();
        root_items.sort_by_key(|item| item.last_touched);
        root_items.reverse();
        root_items
    }

    pub fn children(&self, item: &Item) -> Vec<Scru128Id> {
        let mut children = item.children.clone();
        children.extend(&item.forked_children);
        children.sort_by_key(|child| {
            self.items
                .get(child)
                .map(|item| item.last_touched)
                .unwrap_or_default()
        });
        children.reverse();
        children
    }

    pub fn first(&self) -> Option<&Item> {
        let root = self.root();
        if !root.is_empty() {
            let stack = &root[0];
            let children = self.children(stack);
            let id = if !children.is_empty() {
                children[0]
            } else {
                stack.id
            };
            self.items.get(&id)
        } else {
            None
        }
    }

    pub fn get_peers(&self, item: &Item) -> Vec<&Item> {
        if let Some(stack) = item.stack_id.and_then(|id| self.items.get(&id)) {
            self.children(stack)
                .iter()
                .map(|id| self.items.get(id).unwrap())
                .collect()
        } else {
            self.root()
        }
    }

    pub fn get_best_focus(&self, item: Option<&Item>) -> Option<&Item> {
        if item.is_none() {
            return self.first();
        }

        let item = item.unwrap();
        if let Some(item) = self.items.get(&item.id) {
            return Some(item);
        }

        let peers = self.get_peers(item);
        if peers.is_empty() {
            return item.stack_id.and_then(|id| self.items.get(&id));
        }

        peers
            .iter()
            .position(|peer| peer.last_touched < item.last_touched)
            .map(|position| peers[position])
            .or(Some(peers[peers.len() - 1]))
    }

    pub fn filter(&self, matches: &HashSet<ssri::Integrity>) -> Self {
        let items: HashMap<Scru128Id, Item> = self
            .items
            .values()
            .filter_map(|item| {
                let mut item = item.clone();
                if item.stack_id.is_none() {
                    item.children = self
                        .children(&item)
                        .into_iter()
                        .filter(|child_id| {
                            if let Some(child) = self.items.get(child_id) {
                                matches.contains(&child.hash)
                            } else {
                                false
                            }
                        })
                        .collect();
                    if item.children.is_empty() {
                        return None;
                    }
                } else if !matches.contains(&item.hash) {
                    return None;
                }
                Some((item.id, item))
            })
            .collect();

        View {
            items,
            undo: self.undo.clone(),
        }
    }
}
