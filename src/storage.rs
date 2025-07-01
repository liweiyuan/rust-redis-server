use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Storage {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let db = self.data.lock().unwrap();
        db.get(key).cloned()
    }

    pub fn set(&self, key: String, value: String) {
        let mut db = self.data.lock().unwrap();
        db.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_set_get() {
        let storage = Storage::new();
        storage.set("key1".to_string(), "value1".to_string());
        assert_eq!(storage.get("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_storage_get_non_existent() {
        let storage = Storage::new();
        assert_eq!(storage.get("non_existent_key"), None);
    }

    #[test]
    fn test_storage_overwrite() {
        let storage = Storage::new();
        storage.set("key1".to_string(), "value1".to_string());
        storage.set("key1".to_string(), "new_value".to_string());
        assert_eq!(storage.get("key1"), Some("new_value".to_string()));
    }

    #[test]
    fn test_storage_thread_safety() {
        let storage = Storage::new();
        let storage_clone = storage.clone();

        let handle = std::thread::spawn(move || {
            storage_clone.set("thread_key".to_string(), "thread_value".to_string());
        });
        handle.join().unwrap();

        assert_eq!(storage.get("thread_key"), Some("thread_value".to_string()));
    }
}