use std::collections::{HashMap, HashSet};

use scru128::Scru128Id;
use ssri::Integrity;

use crate::store::{Movement, Packet, PacketType, StackLockStatus, StackSortOrder};

#[derive(serde::Serialize, Debug, Clone)]
pub struct Item {
    pub id: Scru128Id,
    pub last_touched: Scru128Id,
    pub touched: Vec<Scru128Id>,
    pub hash: Integrity,
    pub stack_id: Option<Scru128Id>,
    children: Vec<Scru128Id>,
    pub ephemeral: bool,
    pub ordered: bool,
    pub locked: bool,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct Focus {
    pub item: Item,
    pub index: usize,
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

    pub fn merge(&mut self, packet: &Packet) {
        match packet.packet_type {
            PacketType::Add => {
                // remove potentially old ephemeral versions of this packet
                if let Some(stack) = packet.stack_id.and_then(|id| self.items.get_mut(&id)) {
                    stack.children.retain(|id| id != &packet.id);
                }

                // If this packet isn't ephemeral, check if an item with the same hash already
                // exists in the same stack, in order to avoid duplicates
                if !packet.ephemeral {
                    if let Some(stack) = packet.stack_id.and_then(|id| self.items.get_mut(&id)) {
                        let children = stack.children.clone();
                        for child_id in children {
                            if let Some(child) = self.items.get_mut(&child_id) {
                                if !child.ephemeral && &child.hash == packet.hash.as_ref().unwrap()
                                {
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

                // Otherwise add a new item
                let item = Item {
                    id: packet.id,
                    last_touched: packet.id,
                    touched: vec![packet.id],
                    hash: packet.hash.clone().unwrap(),
                    stack_id: packet.stack_id,
                    children: Vec::new(),
                    ephemeral: packet.ephemeral,
                    ordered: false,
                    locked: match packet.lock_status {
                        Some(StackLockStatus::Locked) => true,
                        _ => false,
                    },
                };

                if let Some(stack) = packet.stack_id.and_then(|id| self.items.get_mut(&id)) {
                    stack.children.push(packet.id);
                    if packet.id > stack.last_touched {
                        stack.last_touched = packet.id;
                    }
                }
                self.items.insert(packet.id, item);
            }

            PacketType::Update => {
                if packet.source_id.is_none() {
                    return;
                }
                let source_id = packet.source_id.unwrap();

                if let Some(movement) = &packet.movement {
                    if let Some(item) = self.items.get(&source_id) {
                        let stack_id = item.stack_id;
                        let item_id = item.id;
                        if let Some(stack) = stack_id.and_then(|id| self.items.get_mut(&id)) {
                            println!("MOVE PACKET: {:?} {:?}", movement, stack);
                            if let Some(index) = stack.children.iter().position(|id| item_id == *id)
                            {
                                match movement {
                                    Movement::Up => {
                                        if index > 0 {
                                            stack.children.swap(index, index - 1);
                                        }
                                    }
                                    Movement::Down => {
                                        if index < stack.children.len() - 1 {
                                            stack.children.swap(index, index + 1);
                                        }
                                    }
                                }
                            }
                            stack.ordered = true;
                        }
                    }
                    return;
                }

                if let Some(sort_order) = &packet.sort_order {
                    if let Some(item) = self.items.get_mut(&source_id) {
                        match sort_order {
                            StackSortOrder::Auto => item.ordered = false,
                            StackSortOrder::Manual => item.ordered = true,
                        }
                    }
                    return;
                }

                if let Some(lock_status) = &packet.lock_status {
                    if let Some(item) = self.items.get_mut(&source_id) {
                        match lock_status {
                            StackLockStatus::Unlocked => item.locked = false,
                            StackLockStatus::Locked => item.locked = true,
                        }
                    }
                    return;
                }

                if let Some(item) = self.items.get(&source_id).cloned() {
                    let mut item = item;

                    if let Some(hash) = &packet.hash {
                        item.hash = hash.clone();
                    }

                    if let Some(new_stack_id) = packet.stack_id {
                        if let Some(old_stack) =
                            item.stack_id.and_then(|id| self.items.get_mut(&id))
                        {
                            old_stack.children.retain(|&id| id != source_id);
                        }
                        item.stack_id = Some(new_stack_id);
                        if let Some(new_stack) = self.items.get_mut(&new_stack_id) {
                            new_stack.children.push(source_id);
                        }
                    }

                    item.touched.push(packet.id);
                    item.last_touched = packet.id;
                    if let Some(stack) = item.stack_id.and_then(|id| self.items.get_mut(&id)) {
                        stack.last_touched = packet.id;
                    }

                    self.items.insert(source_id, item);
                }
            }

            PacketType::Fork => {
                let source_id = packet.source_id.unwrap();

                if let Some(item) = self.items.get(&source_id) {
                    assert!(
                        item.stack_id.is_some(),
                        "Forking Stacks is not supported yet"
                    );

                    let mut new_item = item.clone();
                    new_item.id = packet.id;

                    new_item.children = Vec::new();

                    if let Some(hash) = &packet.hash {
                        new_item.hash = hash.clone();
                    }

                    if let Some(new_stack_id) = packet.stack_id {
                        new_item.stack_id = Some(new_stack_id);
                    }

                    new_item.touched.push(packet.id);
                    new_item.last_touched = packet.id;

                    if let Some(stack) = new_item.stack_id.and_then(|id| self.items.get_mut(&id)) {
                        // And add the new item to children
                        stack.children.push(packet.id);
                        stack.last_touched = packet.id;
                    }

                    self.items.insert(packet.id, new_item);
                }
            }

            PacketType::Delete => {
                let source_id = packet.source_id.unwrap();
                if let Some(mut item) = self.items.remove(&source_id) {
                    if let Some(stack) = item.stack_id.and_then(|id| self.items.get_mut(&id)) {
                        stack.children.retain(|&id| id != source_id);
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
        if item.ordered {
            return children;
        }
        children.sort_by_key(|child| {
            self.items
                .get(child)
                .map(|item| item.last_touched)
                .unwrap_or_default()
        });
        children.reverse();
        children
    }

    pub fn first(&self) -> Option<Focus> {
        let root = self.root();
        if !root.is_empty() {
            let stack = &root[0];
            let children = self.children(stack);
            let id = if !children.is_empty() {
                children[0]
            } else {
                stack.id
            };
            self.items.get(&id).and_then(|item| {
                Some(Focus {
                    item: item.clone(),
                    index: 0,
                })
            })
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

    pub fn get_focus_for_id(&self, id: &Scru128Id) -> Option<Focus> {
        self.items.get(id).and_then(|item| {
            let peers = self.get_peers(&item);
            peers
                .iter()
                .position(|&peer| item.id == peer.id)
                .and_then(|index| {
                    Some(Focus {
                        item: item.clone(),
                        index,
                    })
                })
        })
    }

    pub fn get_best_focus_with_offset(&self, focus: &Option<Focus>, offset: i8) -> Option<Focus> {
        if focus.is_none() {
            return self.first();
        }
        let focus = focus.as_ref().unwrap();

        let item = self.items.get(&focus.item.id).unwrap_or(&focus.item);
        let peers = self.get_peers(item);

        if peers.is_empty() {
            return item.stack_id.and_then(|id| self.get_focus_for_id(&id));
        }

        let mut idx = peers
            .iter()
            .position(|&peer| item.id == peer.id)
            .unwrap_or(focus.index);

        if offset.is_negative() {
            idx = idx.saturating_sub((-offset) as usize)
        } else {
            idx = idx.saturating_add(offset as usize)
        };

        if idx >= peers.len() {
            idx = peers.len() - 1;
        }
        return Some(Focus {
            item: peers[idx].clone(),
            index: idx,
        });
    }

    pub fn get_best_focus(&self, focus: &Option<Focus>) -> Option<Focus> {
        self.get_best_focus_with_offset(focus, 0)
    }

    pub fn get_best_focus_next(&self, focus: &Option<Focus>) -> Option<Focus> {
        self.get_best_focus_with_offset(focus, 1)
    }

    pub fn get_best_focus_prev(&self, focus: &Option<Focus>) -> Option<Focus> {
        self.get_best_focus_with_offset(focus, -1)
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
                                return matches.contains(&child.hash);
                            }
                            false
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
