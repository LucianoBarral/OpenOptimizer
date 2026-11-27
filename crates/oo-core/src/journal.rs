use crate::model::{Change, OoError};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Journal append-only em JSONL. Cada linha é um snapshot completo de uma Change;
/// ao carregar, a última linha de cada id vence.
pub struct Journal {
    path: PathBuf,
    order: Vec<String>,
    latest: HashMap<String, Change>,
}

impl Journal {
    pub fn open(path: PathBuf) -> Result<Self, OoError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut j = Journal {
            path,
            order: vec![],
            latest: HashMap::new(),
        };
        if let Ok(f) = File::open(&j.path) {
            for line in BufReader::new(f).lines() {
                let Ok(line) = line else { break };
                // Linha truncada (crash no meio da escrita) não desserializa e é descartada.
                if let Ok(c) = serde_json::from_str::<Change>(&line) {
                    j.fold(c);
                }
            }
        }
        Ok(j)
    }

    fn fold(&mut self, c: Change) {
        if !self.latest.contains_key(&c.id) {
            self.order.push(c.id.clone());
        }
        self.latest.insert(c.id.clone(), c);
    }

    /// Write-ahead: só retorna depois de o dado estar no disco (sync_all = FlushFileBuffers).
    pub fn record(&mut self, c: &Change) -> Result<(), OoError> {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let mut line = serde_json::to_string(c).map_err(|e| OoError::Io(e.to_string()))?;
        line.push('\n');
        f.write_all(line.as_bytes())?;
        f.sync_all()?;
        self.fold(c.clone());
        Ok(())
    }

    pub fn changes(&self) -> Vec<Change> {
        self.order
            .iter()
            .map(|id| self.latest[id].clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use std::io::Write;

    fn change(id: &str, status: ChangeStatus) -> Change {
        Change {
            id: id.into(),
            tweak_id: "t".into(),
            batch_id: "b".into(),
            timestamp: "2026-11-27T10:00:00Z".into(),
            steps: vec![],
            status,
            post: None,
        }
    }

    #[test]
    fn later_record_wins_and_order_is_stable() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("j.jsonl");
        let mut j = Journal::open(p.clone()).unwrap();
        j.record(&change("a", ChangeStatus::Pending)).unwrap();
        j.record(&change("b", ChangeStatus::Pending)).unwrap();
        j.record(&change("a", ChangeStatus::Applied)).unwrap();
        let c = Journal::open(p).unwrap().changes();
        assert_eq!(
            c.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        assert_eq!(c[0].status, ChangeStatus::Applied);
    }

    #[test]
    fn truncated_last_line_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("j.jsonl");
        Journal::open(p.clone())
            .unwrap()
            .record(&change("a", ChangeStatus::Applied))
            .unwrap();
        std::fs::OpenOptions::new()
            .append(true)
            .open(&p)
            .unwrap()
            .write_all(b"{\"id\":\"tru")
            .unwrap();
        assert_eq!(Journal::open(p).unwrap().changes().len(), 1);
    }
}
