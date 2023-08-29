use std::collections::HashMap;

use scru128::Scru128Id;
use ssri::Integrity;

use crate::store::Packet;

#[derive(Debug, Clone)]
pub struct Item {
    pub id: Scru128Id,
    pub last_touched: Scru128Id,
    pub touched: Vec<Scru128Id>,
    pub hash: Integrity,
    pub stack_id: Option<Scru128Id>,
    pub children: Vec<Scru128Id>,
    pub forked_children: Vec<Scru128Id>,
}

#[derive(Debug, Clone)]
pub struct View {
    pub items: HashMap<Scru128Id, Item>,
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
                if let Some(item) = self.items.remove(&packet.source_id) {
                    if let Some(stack) = item.stack_id.and_then(|id| self.items.get_mut(&id)) {
                        stack.children.retain(|&id| id != packet.source_id);
                        stack.last_touched = packet.id;
                    }
                }
            }
        }
    }

    pub fn root(&self) -> Vec<Item> {
        let mut root_items = self
            .items
            .values()
            .filter(|item| item.stack_id.is_none())
            .cloned()
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

    pub fn get_peers(&self, focused_id: &Scru128Id) -> Vec<Scru128Id> {
        if let Some(item) = self.items.get(focused_id) {
            if let Some(stack_id) = item.stack_id {
                self.children(&self.items[&stack_id])
            } else {
                self.root().iter().map(|item| item.id).collect()
            }
        } else {
            Vec::new()
        }
    }
}
