use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;

use serde::{Deserialize, Serialize};

use lmdb::Cursor;
use lmdb::Transaction;

// POLL_INTERVAL is the number of milliseconds to wait between polls when watching for
// additions to the stream
// todo: investigate switching to: https://docs.rs/notify/latest/notify/
pub const POLL_INTERVAL: u64 = 5;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub id: scru128::Scru128Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseFrame {
    source_id: scru128::Scru128Id,
    data: String,
}

#[derive(Debug, PartialEq)]
struct Event {
    data: String,
    event: Option<String>,
    id: Option<i64>,
}

#[allow(dead_code)]
fn parse_sse<R: Read>(buf: &mut BufReader<R>) -> Option<Event> {
    let mut line = String::new();

    let mut data = Vec::<String>::new();
    let mut id: Option<i64> = None;

    loop {
        line.clear();
        let n = buf.read_line(&mut line).unwrap();
        if n == 0 {
            // stream interrupted
            return None;
        }

        if line == "\n" {
            // end of event, emit
            break;
        }

        let (field, rest) = line.split_at(line.find(":").unwrap() + 1);
        let rest = rest.trim();
        match field {
            // comment
            ":" => (),
            "id:" => id = Some(rest.parse::<i64>().unwrap()),
            "data:" => data.push(rest.to_string()),
            _ => todo!(),
        };
    }

    return Some(Event {
        data: data.join(" "),
        event: None,
        id: id,
    });
}

pub fn store_open(path: &std::path::Path) -> lmdb::Environment {
    std::fs::create_dir_all(path).unwrap();
    let env = lmdb::Environment::new()
        .set_map_size(10 * 10485760)
        .open(path)
        .unwrap();
    return env;
}

pub fn store_put(
    env: &lmdb::Environment,
    topic: Option<String>,
    attribute: Option<String>,
    data: String,
) -> scru128::Scru128Id {
    let id = scru128::new();

    let frame = Frame {
        id: id,
        topic: topic.clone(),
        attribute: attribute.clone(),
        data: data.trim().to_string(),
    };
    let frame = serde_json::to_vec(&frame).unwrap();

    let db = env.open_db(None).unwrap();
    let mut txn = env.begin_rw_txn().unwrap();
    txn.put(
        db,
        // if I understand the docs right, this should be 'to_ne_bytes', but that doesn't
        // work
        &id.to_u128().to_be_bytes(),
        &frame,
        lmdb::WriteFlags::empty(),
    )
    .unwrap();
    txn.commit().unwrap();

    return id;
}

#[allow(dead_code)]
fn store_get(env: &lmdb::Environment, id: scru128::Scru128Id) -> Option<Frame> {
    let db = env.open_db(None).unwrap();
    let txn = env.begin_ro_txn().unwrap();
    match txn.get(db, &id.to_u128().to_be_bytes()) {
        Ok(value) => Some(serde_json::from_slice(&value).unwrap()),
        Err(lmdb::Error::NotFound) => None,
        Err(err) => panic!("store_get: {:?}", err),
    }
}

pub fn store_cat(env: &lmdb::Environment, last_id: Option<scru128::Scru128Id>) -> Vec<Frame> {
    let db = env.open_db(None).unwrap();
    let txn = env.begin_ro_txn().unwrap();
    let mut c = txn.open_ro_cursor(db).unwrap();
    let it = match last_id {
        Some(key) => {
            let mut i = c.iter_from(&key.to_u128().to_be_bytes());
            i.next();
            i
        }
        None => c.iter_start(),
    };
    it.map(|item| -> Frame {
        let (_, value) = item.unwrap();
        serde_json::from_slice(&value).unwrap()
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use temp_dir::TempDir;
    // use pretty_assertions::assert_eq;

    #[test]
    fn test_store() {
        let d = TempDir::new().unwrap();
        let env = store_open(&d.path());

        let id = store_put(&env, None, None, "foo".into());
        assert_eq!(store_cat(&env, None).len(), 1);

        let frame = store_get(&env, id).unwrap();
        assert_eq!(
            frame,
            Frame {
                id: id,
                topic: None,
                attribute: None,
                data: "foo".into()
            }
        );

        // skip with last_id
        assert_eq!(store_cat(&env, Some(id)).len(), 0);
    }

    #[test]
    fn test_parse_sse() {
        let mut buf = BufReader::new(
            indoc! {"
        : welcome
        id: 1
        data: foo
        data: bar

        id: 2
        data: hai

        "}
            .as_bytes(),
        );

        let event = parse_sse(&mut buf).unwrap();
        assert_eq!(
            event,
            Event {
                data: "foo bar".into(),
                event: None,
                id: Some(1),
            }
        );

        let event = parse_sse(&mut buf).unwrap();
        assert_eq!(
            event,
            Event {
                data: "hai".into(),
                event: None,
                id: Some(2),
            }
        );
    }
}
