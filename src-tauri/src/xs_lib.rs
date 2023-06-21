use serde::{Deserialize, Serialize};

use lmdb::Cursor;
use lmdb::Transaction;

#[derive(Debug, PartialEq)]
struct Event {
    data: String,
    event: Option<String>,
    id: Option<i64>,
}

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
pub struct ResponseFrame {
    pub source_id: scru128::Scru128Id,
    pub data: String,
}

pub fn store_open(path: &std::path::Path) -> Result<lmdb::Environment, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(path)?;
    let env = lmdb::Environment::new()
        .set_map_size(10 * 10485760)
        .open(path)?;
    Ok(env)
}

pub fn store_put(
    env: &lmdb::Environment,
    topic: Option<String>,
    attribute: Option<String>,
    data: String,
) -> Result<scru128::Scru128Id, Box<dyn std::error::Error>> {
    let id = scru128::new();

    let frame = Frame {
        id,
        topic,
        attribute,
        data: data.trim().to_string(),
    };
    let frame = serde_json::to_vec(&frame)?;

    let db = env.open_db(None)?;
    let mut txn = env.begin_rw_txn()?;
    txn.put(
        db,
        &id.to_u128().to_be_bytes(),
        &frame,
        lmdb::WriteFlags::empty(),
    )?;
    txn.commit()?;

    Ok(id)
}

#[allow(dead_code)]
fn store_get(
    env: &lmdb::Environment,
    id: scru128::Scru128Id,
) -> Result<Option<Frame>, Box<dyn std::error::Error>> {
    let db = env.open_db(None)?;
    let txn = env.begin_ro_txn()?;
    match txn.get(db, &id.to_u128().to_be_bytes()) {
        Ok(value) => Ok(Some(serde_json::from_slice(value)?)),
        Err(lmdb::Error::NotFound) => Ok(None),
        Err(err) => Err(Box::new(err)),
    }
}

pub fn store_delete(
    env: &lmdb::Environment,
    ids: Vec<scru128::Scru128Id>,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = env.open_db(None)?;
    let mut txn = env.begin_rw_txn()?;
    for id in ids {
        txn.del(db, &id.to_u128().to_be_bytes(), None)?;
    }
    txn.commit()?;
    Ok(())
}

pub fn store_cat(
    env: &lmdb::Environment,
    last_id: Option<scru128::Scru128Id>,
) -> Result<Vec<Frame>, Box<dyn std::error::Error>> {
    let db = env.open_db(None)?;
    let txn = env.begin_ro_txn()?;
    let mut c = txn.open_ro_cursor(db)?;
    let it: Box<dyn Iterator<Item = Result<(&[u8], &[u8]), lmdb::Error>>> = match last_id {
        Some(key) => {
            let key_bytes = key.to_u128().to_be_bytes();
            Box::new(c.iter_from(key_bytes).filter_map(move |item| {
                if item.as_ref().map(|(k, _)| *k == key_bytes).unwrap_or(false) {
                    None
                } else {
                    Some(item)
                }
            }))
        }
        None => Box::new(c.iter_start()),
    };
    it.map(|item| -> Result<Frame, Box<dyn std::error::Error>> {
        let (_, value) = item?;
        Ok(serde_json::from_slice(value)?)
    })
    .collect::<Result<Vec<_>, _>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::io::BufRead;
    use std::io::Read;
    use temp_dir::TempDir;
    // use pretty_assertions::assert_eq;

    #[test]
    fn test_store() {
        let d = TempDir::new().unwrap();
        let env = store_open(&d.path()).unwrap();

        let id = store_put(&env, None, None, "foo".into()).unwrap();
        assert_eq!(store_cat(&env, None).unwrap().len(), 1);

        let frame = store_get(&env, id).unwrap().unwrap();
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
        assert_eq!(store_cat(&env, Some(id)).unwrap().len(), 0);
    }

    #[test]
    fn test_cat_after_delete() {
        let d = TempDir::new().unwrap();
        let env = store_open(&d.path()).unwrap();

        let _ = store_put(&env, None, None, "1".into()).unwrap();
        let _ = store_put(&env, None, None, "2".into()).unwrap();
        let id = store_put(&env, None, None, "3".into()).unwrap();
        assert_eq!(store_cat(&env, Some(id)).unwrap().len(), 0);

        store_delete(&env, vec![id]).unwrap();
        let _ = store_put(&env, None, None, "4".into()).unwrap();
        assert_eq!(store_cat(&env, Some(id)).unwrap().len(), 1);
    }

    use std::io::BufReader;
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
