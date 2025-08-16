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
    let update_packet = store.update(packet.id, Some(updated_content), MimeType::TextPlain, None);

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
    let forked_packet = store.fork(packet.id, Some(forked_content), MimeType::TextPlain, None);

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
    let delete_packet = store.delete(packet.id);
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

    let results = store.index.query("fuzzy", None).unwrap();
    let results: Vec<_> = results
        .into_iter()
        .map(|(hash, _score)| store.cas_read(&hash).unwrap())
        .collect();
    assert_eq!(results, vec![b"Hello, fuzzy world!".to_vec()]);
}

#[test]
fn test_is_valid_https_url() {
    assert!(is_valid_https_url(b"https://www.example.com"));
    assert!(!is_valid_https_url(b"Good afternoon"));
}

#[test]
fn test_purge() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut store = Store::new(temp_dir.path().to_str().unwrap());

    let content = b"SECRET_KEY=super_secret_value";
    let stack_id = scru128::new();
    let packet = store.add(content, MimeType::TextPlain, stack_id);
    let hash = packet.hash.clone().unwrap();

    // Verify content exists before purge
    assert!(store.cas_read(&hash).is_some());
    assert!(store.get_content_meta(&hash).is_some());

    // Purge the content
    store.purge(&hash).unwrap();

    // Verify content is gone after purge
    assert!(store.cas_read(&hash).is_none());
    assert!(store.get_content_meta(&hash).is_none());

    // Test that scan_content_meta skips missing content
    let content_meta_cache = store.scan_content_meta();
    assert!(!content_meta_cache.contains_key(&hash));

    // Add some new content to verify the store still works
    let new_content = b"This is safe content";
    let new_packet = store.add(new_content, MimeType::TextPlain, stack_id);
    let new_hash = new_packet.hash.unwrap();

    assert!(store.cas_read(&new_hash).is_some());
    assert!(store.get_content_meta(&new_hash).is_some());
}

#[test]
fn test_enumerate_cas() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut store = Store::new(temp_dir.path().to_str().unwrap());

    let stack_id = scru128::new();

    // Add some content
    let content1 = b"First content";
    let packet1 = store.add(content1, MimeType::TextPlain, stack_id);
    let hash1 = packet1.hash.unwrap();

    let content2 = b"Second content";
    let packet2 = store.add(content2, MimeType::TextPlain, stack_id);
    let hash2 = packet2.hash.unwrap();

    // Enumerate CAS entries
    let cas_hashes = store.enumerate_cas();

    // Should contain both hashes
    assert!(cas_hashes.contains(&hash1));
    assert!(cas_hashes.contains(&hash2));
    assert_eq!(cas_hashes.len(), 2);

    // Purge one entry
    store.purge(&hash1).unwrap();

    // Enumerate again - should only have one hash now
    let cas_hashes_after_purge = store.enumerate_cas();
    assert!(!cas_hashes_after_purge.contains(&hash1));
    assert!(cas_hashes_after_purge.contains(&hash2));
    assert_eq!(cas_hashes_after_purge.len(), 1);
}

#[test]
fn test_index_deletion() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let mut store = Store::new(path);

    // Add test content that will be indexed
    let test_content = b"unique_test_content_for_deletion_123";
    let hash = store.cas_write(test_content, MimeType::TextPlain, "Text".to_string());

    println!("Added content with hash: {hash}");

    // Verify content is searchable before deletion
    let search_results_before = store.index.query("unique_test_content", Some(10)).unwrap();
    println!(
        "Search results before deletion: {} matches",
        search_results_before.len()
    );

    // Verify our specific hash is in the search results
    let found_before = search_results_before.iter().any(|(h, _score)| h == &hash);
    assert!(
        found_before,
        "Content should be found in search before deletion"
    );

    // Now purge the content (this should remove it from the index)
    println!("Purging content with hash: {hash}");
    store.purge(&hash).expect("Purge should succeed");

    // Verify content is no longer searchable after deletion
    let search_results_after = store.index.query("unique_test_content", Some(10)).unwrap();
    println!(
        "Search results after deletion: {} matches",
        search_results_after.len()
    );

    // Verify our specific hash is no longer in the search results
    let found_after = search_results_after.iter().any(|(h, _score)| h == &hash);
    assert!(
        !found_after,
        "Content should NOT be found in search after deletion"
    );

    // Additional verification: try to read the content (should fail)
    let content_after_purge = store.cas_read(&hash);
    assert!(
        content_after_purge.is_none(),
        "Content should not be readable after purge"
    );
}

#[test]
fn test_rebuild_index() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let mut store = Store::new(path);

    // Add multiple test items with different content types
    let text_content1 = b"first searchable content item";
    let text_content2 = b"second searchable content item";
    let text_content3 = b"third different content";

    let hash1 = store.cas_write(text_content1, MimeType::TextPlain, "Text".to_string());
    let hash2 = store.cas_write(text_content2, MimeType::TextPlain, "Text".to_string());
    let hash3 = store.cas_write(text_content3, MimeType::TextPlain, "Text".to_string());

    println!("Added 3 text items to store and index");

    // Verify all content is searchable before rebuild
    let search_results_before = store.index.query("searchable", Some(10)).unwrap();
    assert_eq!(
        search_results_before.len(),
        2,
        "Should find 2 items with 'searchable'"
    );

    let search_results_content = store.index.query("content", Some(10)).unwrap();
    assert_eq!(
        search_results_content.len(),
        3,
        "Should find 3 items with 'content'"
    );

    // Call rebuild_index
    println!("Rebuilding index...");
    let (total_items, indexed_count) = store.rebuild_index().unwrap();

    println!("Rebuild complete: {total_items} total items, {indexed_count} indexed");
    assert_eq!(total_items, 3, "Should have 3 total CAS items");
    assert_eq!(indexed_count, 3, "Should have indexed 3 text items");

    // Verify content is still searchable after rebuild
    let search_results_after = store.index.query("searchable", Some(10)).unwrap();
    assert_eq!(
        search_results_after.len(),
        2,
        "Should still find 2 items with 'searchable' after rebuild"
    );

    let search_results_content_after = store.index.query("content", Some(10)).unwrap();
    assert_eq!(
        search_results_content_after.len(),
        3,
        "Should still find 3 items with 'content' after rebuild"
    );

    // Verify specific hashes are still found
    let found_hash1 = search_results_content_after
        .iter()
        .any(|(h, _)| h == &hash1);
    let found_hash2 = search_results_content_after
        .iter()
        .any(|(h, _)| h == &hash2);
    let found_hash3 = search_results_content_after
        .iter()
        .any(|(h, _)| h == &hash3);

    assert!(found_hash1, "Hash1 should be found after rebuild");
    assert!(found_hash2, "Hash2 should be found after rebuild");
    assert!(found_hash3, "Hash3 should be found after rebuild");
}

#[test]
fn test_rebuild_index_with_mixed_content() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let mut store = Store::new(path);

    // Add text content that gets indexed
    let text_content = b"searchable text content";
    let text_hash = store.cas_write(text_content, MimeType::TextPlain, "Text".to_string());

    // Add image content that doesn't get indexed
    let image_content = b"fake image bytes";
    let image_hash = store.cas_write(image_content, MimeType::ImagePng, "Image".to_string());

    // Verify only text content is searchable
    let search_before = store.index.query("searchable", Some(10)).unwrap();
    assert_eq!(search_before.len(), 1, "Should find 1 text item");

    // Rebuild the index
    let (total_items, indexed_count) = store.rebuild_index().unwrap();
    assert_eq!(total_items, 2, "Should have 2 total CAS items");
    assert_eq!(indexed_count, 1, "Should have indexed only 1 text item");

    // Verify text content is still searchable after rebuild
    let search_after = store.index.query("searchable", Some(10)).unwrap();
    assert_eq!(
        search_after.len(),
        1,
        "Should still find 1 text item after rebuild"
    );

    // Verify the right hash is found
    let found_text = search_after.iter().any(|(h, _)| h == &text_hash);
    assert!(found_text, "Should find the text content hash");

    // Verify we can still read both items from CAS
    assert!(
        store.cas_read(&text_hash).is_some(),
        "Text content should still be readable"
    );
    assert!(
        store.cas_read(&image_hash).is_some(),
        "Image content should still be readable"
    );
}
