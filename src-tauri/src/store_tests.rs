use crate::store::{is_valid_https_url, MimeType, Packet, PacketType, StackLockStatus, Store};

use tempfile::tempdir;

#[test]
fn test_add() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let mut store = Store::new(path);

    let content = b"Hello, world!";
    let packet = store.add_stack(content, StackLockStatus::Unlocked);

    let stored_packet = store.scan().next().unwrap();
    assert_eq!(packet, stored_packet);

    match packet.packet_type {
        PacketType::Add => {
            let stored_content = store.cas_read(&packet.hash.unwrap()).unwrap();
            assert_eq!(content.to_vec(), stored_content);
        }
        _ => panic!("Expected AddPacket"),
    }
}

#[test]
fn test_update() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let mut store = Store::new(path);

    let content = b"Hello, world!";
    let packet = store.add_stack(content, StackLockStatus::Unlocked);

    let updated_content = b"Hello, updated world!";
    let update_packet = store.update(
        packet.id.clone(),
        Some(updated_content),
        MimeType::TextPlain,
        None,
    );

    let stored_update_packet = store.scan().last().unwrap();
    assert_eq!(update_packet, stored_update_packet);

    match stored_update_packet {
        Packet {
            packet_type: PacketType::Update,
            hash: Some(hash),
            ..
        } => {
            let stored_content = store.cas_read(&hash).unwrap();
            assert_eq!(updated_content.to_vec(), stored_content);
        }
        _ => panic!("Expected UpdatePacket"),
    }
}

#[test]
fn test_fork() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let mut store = Store::new(path);

    let content = b"Hello, world!";
    let packet = store.add_stack(content, StackLockStatus::Unlocked);

    let forked_content = b"Hello, forked world!";
    let forked_packet = store.fork(
        packet.id.clone(),
        Some(forked_content),
        MimeType::TextPlain,
        None,
    );

    let stored_fork_packet = store.scan().last().unwrap();
    assert_eq!(forked_packet, stored_fork_packet);

    match forked_packet {
        Packet {
            packet_type: PacketType::Fork,
            hash,
            ..
        } => {
            let stored_content = store.cas_read(&hash.unwrap()).unwrap();
            assert_eq!(forked_content.to_vec(), stored_content);
        }
        _ => panic!("Expected ForkPacket"),
    }
}

#[test]
fn test_delete() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let mut store = Store::new(path);
    let content = b"Hello, world!";
    let packet = store.add_stack(content, StackLockStatus::Unlocked);
    let delete_packet = store.delete(packet.id.clone());
    let stored_delete_packet = store.scan().last().unwrap();
    assert_eq!(delete_packet, stored_delete_packet);
}

#[test]
fn test_query() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let mut store = Store::new(path);

    let content1 = b"Hello, world!";
    let content2 = b"Hello, fuzzy world!";
    let content3 = b"Hello, there!";

    store.add_stack(content1, StackLockStatus::Unlocked);
    store.add_stack(content2, StackLockStatus::Unlocked);
    store.add_stack(content3, StackLockStatus::Unlocked);

    let results = store.index.query("fzzy");
    let results: Vec<_> = results
        .into_iter()
        .map(|hash| store.cas_read(&hash).unwrap())
        .collect();
    assert_eq!(results, vec![b"Hello, fuzzy world!".to_vec()]);
}

#[test]
fn test_is_valid_https_url() {
    assert!(is_valid_https_url(b"https://www.example.com"));
    assert!(!is_valid_https_url(b"Good afternoon"));
}
