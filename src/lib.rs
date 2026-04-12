#![allow(dead_code)]
pub struct Entry { key: String, value: String, version: u32, created: u64, modified: u64, ttl: Option<u64> }
pub struct Memory { entries: Vec<Entry>, snapshots: Vec<Vec<Entry>> }
impl Memory {
    pub fn new() -> Self { Self { entries: Vec::new(), snapshots: Vec::new() } }
    pub fn set(&mut self, key: &str, value: &str, ttl: Option<u64>, now: u64) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.key == key) { e.value = value.to_string(); e.version += 1; e.modified = now; if let Some(t) = ttl { e.ttl = Some(now + t); } }
        else { self.entries.push(Entry { key: key.to_string(), value: value.to_string(), version: 1, created: now, modified: now, ttl: ttl.map(|t| now + t) }); }
    }
    pub fn get(&self, key: &str) -> Option<&str> { self.entries.iter().find(|e| e.key == key).map(|e| e.value.as_str()) }
    pub fn delete(&mut self, key: &str) -> bool { let n = self.entries.len(); self.entries.retain(|e| e.key != key); self.entries.len() < n }
    pub fn exists(&self, key: &str) -> bool { self.entries.iter().any(|e| e.key == key) }
    pub fn version_of(&self, key: &str) -> u32 { self.entries.iter().find(|e| e.key == key).map(|e| e.version).unwrap_or(0) }
    pub fn expire(&mut self, now: u64) -> Vec<String> {
        let expired: Vec<String> = self.entries.iter().filter(|e| e.ttl.map_or(false, |t| now >= t)).map(|e| e.key.clone()).collect();
        self.entries.retain(|e| !e.ttl.map_or(false, |t| now >= t)); expired
    }
    pub fn snapshot(&mut self) { self.snapshots.push(self.entries.clone()); }
    pub fn restore(&mut self, index: usize) -> bool { if index < self.snapshots.len() { self.entries = self.snapshots[index].clone(); true } else { false } }
    pub fn snapshot_count(&self) -> usize { self.snapshots.len() }
    pub fn keys(&self) -> Vec<&str> { self.entries.iter().map(|e| e.key.as_str()).collect() }
    pub fn count(&self) -> usize { self.entries.len() }
    pub fn search(&self, prefix: &str) -> Vec<&str> { self.entries.iter().filter(|e| e.key.starts_with(prefix)).map(|e| e.key.as_str()).collect() }
    pub fn age_of(&self, key: &str, now: u64) -> Option<u64> { self.entries.iter().find(|e| e.key == key).map(|e| now - e.created) }
    pub fn clear(&mut self) { self.entries.clear(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let m = Memory::new(); assert_eq!(m.count(), 0); }
    #[test] fn test_set_get() { let mut m = Memory::new(); m.set("k", "v", None, 100); assert_eq!(m.get("k"), Some("v")); }
    #[test] fn test_delete() { let mut m = Memory::new(); m.set("k", "v", None, 100); assert!(m.delete("k")); assert!(!m.exists("k")); }
    #[test] fn test_version() { let mut m = Memory::new(); m.set("k", "v1", None, 100); m.set("k", "v2", None, 200); assert_eq!(m.version_of("k"), 2); }
    #[test] fn test_ttl() { let mut m = Memory::new(); m.set("k", "v", Some(10), 100); let exp = m.expire(111); assert_eq!(exp.len(), 1); assert_eq!(m.count(), 0); }
    #[test] fn test_snapshot_restore() { let mut m = Memory::new(); m.set("a", "1", None, 100); m.snapshot(); m.set("b", "2", None, 200); assert_eq!(m.count(), 2); assert!(m.restore(0)); assert_eq!(m.count(), 1); }
    #[test] fn test_search() { let mut m = Memory::new(); m.set("usr:1", "a", None, 100); m.set("usr:2", "b", None, 100); m.set("sys:x", "c", None, 100); assert_eq!(m.search("usr:").len(), 2); }
    #[test] fn test_age() { let mut m = Memory::new(); m.set("k", "v", None, 100); assert_eq!(m.age_of("k", 200), Some(100)); }
    #[test] fn test_clear() { let mut m = Memory::new(); m.set("a", "b", None, 100); m.clear(); assert_eq!(m.count(), 0); }
    #[test] fn test_keys() { let mut m = Memory::new(); m.set("a", "1", None, 100); m.set("b", "2", None, 100); assert_eq!(m.keys().len(), 2); }
    #[test] fn test_restore_bad_index() { let mut m = Memory::new(); assert!(!m.restore(99)); }
}