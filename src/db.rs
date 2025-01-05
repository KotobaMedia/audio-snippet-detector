/**
 * The database module contains a simple in-memory database of audio fingerprints.
 */
use ndarray::{s, Array1, Array2, ArrayView1, ArrayView2};

fn cosine_similarity(u: &ArrayView1<f32>, v: &ArrayView1<f32>) -> f32 {
    // Dot product
    let dot = u.dot(v);

    // Norms (L2)
    let norm_u = u.mapv(|x| x * x).sum().sqrt();
    let norm_v = v.mapv(|x| x * x).sum().sqrt();

    dot / (norm_u * norm_v)
}

fn fingerprint_match(needle: &ArrayView2<f32>, haystack: &ArrayView2<f32>) -> f32 {
    // The needle may be shorter than the haystack, so we need to slide it along the haystack
    // and compute the cosine similarity at each position.
    let mut best_score = 0.0;

    // println!(
    //     "needle: {:?}, haystack: {:?}",
    //     needle.shape(),
    //     haystack.shape()
    // );

    let flat_needle = Array1::from_iter(needle.iter().cloned());
    // let flat_haystack = Array1::from_iter(haystack.iter().cloned());

    // We'll slide the needle along the haystack in increments of the needle length / 2
    let step = needle.shape()[0] / 2;
    for i in (0..haystack.shape()[0] - needle.shape()[0]).step_by(step) {
        let window = haystack.slice(s![i..i + needle.shape()[0], ..]);
        let flat_window = Array1::from_iter(window.iter().cloned());

        let score = cosine_similarity(&flat_needle.view(), &flat_window.view());
        if score > best_score {
            best_score = score;
        }
    }
    best_score
}

#[derive(Debug)]
pub struct QueryResult {
    pub label: String,
    pub score: f32,
}

pub struct DatabaseEntry {
    pub label: String,

    /// 2D array representing the fingerprint (25ms frames x 20 MFCC features)
    pub fingerprint: Array2<f32>,
}

pub struct Database {
    entries: Vec<DatabaseEntry>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, label: String, fingerprint: ArrayView2<f32>) {
        self.entries.push(DatabaseEntry {
            label,
            fingerprint: fingerprint.to_owned(),
        });
    }

    pub fn query(&self, query: ArrayView2<f32>) -> Option<QueryResult> {
        let mut best_score = 0.0;
        let mut best_label = None;

        for entry in &self.entries {
            let score = fingerprint_match(&query, &entry.fingerprint.view());
            if score > best_score {
                best_score = score;
                best_label = Some(QueryResult {
                    label: entry.label.clone(),
                    score,
                });
            }
        }

        best_label
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ndarray::arr2;

    #[ignore = "WIP: use real data"]
    #[test]
    fn test_database() {
        let mut db = Database::new();
        let fingerprint = arr2(&[[1.0, 2.0, 3.0], [1.0, 2.0, 3.0]]);
        db.insert("test".to_string(), fingerprint.view());

        let fingerprint2 = arr2(&[[3.0, 3.0, 3.0], [1.0, 2.0, 3.0]]);
        db.insert("test2".to_string(), fingerprint2.view());

        let query = arr2(&[[1.0, 2.0, 3.0], [1.0, 2.0, 3.0]]);
        let result = db.query(query.view()).unwrap();
        assert_eq!(result.label, "test");
        assert!(result.score > 0.99);
    }
}
