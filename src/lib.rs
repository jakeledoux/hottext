use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::Path;

type LinePairs = HashMap<String, HashSet<String>>;

pub struct HotText<R: Rng> {
    line_pairs: LinePairs,
    rng: R,
}

impl<R: Rng> HotText<R> {
    pub fn new(rng: R) -> Self {
        HotText {
            line_pairs: HashMap::new(),
            rng,
        }
    }

    pub fn insert(&mut self, key: &str, line: &str) -> Result<(), Box<dyn Error>> {
        if let Some(lines) = self.line_pairs.get_mut(key) {
            lines.insert(line.to_owned());
        } else {
            let mut lines = HashSet::new();
            lines.insert(line.to_string());
            self.line_pairs.insert(key.to_string(), lines);
        }
        Ok(())
    }

    pub fn extend(&mut self, key: &str, new_lines: HashSet<String>) -> Result<(), Box<dyn Error>> {
        if let Some(lines) = self.line_pairs.get_mut(key) {
            lines.extend(new_lines);
        } else {
            self.line_pairs.insert(key.to_string(), new_lines);
        }
        Ok(())
    }

    pub fn load_hashmap(&mut self, line_pairs: LinePairs) -> Result<(), Box<dyn Error>> {
        for (key, new_lines) in line_pairs {
            if let Some(lines) = self.line_pairs.get_mut(&key) {
                lines.extend(new_lines);
            } else {
                self.line_pairs.insert(key, new_lines);
            }
        }
        Ok(())
    }

    pub fn load_json<P: AsRef<Path>>(&mut self, file: P) -> Result<(), Box<dyn Error>> {
        let file = fs::File::open(file)?;
        let line_pairs: LinePairs = serde_json::from_reader(file)?;
        self.load_hashmap(line_pairs)
    }

    pub fn load_toml<P: AsRef<Path>>(&mut self, file: P) -> Result<(), Box<dyn Error>> {
        let content = fs::read_to_string(file)?;
        let line_pairs: LinePairs = toml::from_str(&content)?;
        self.load_hashmap(line_pairs)
    }

    pub fn with_load_json<P: AsRef<Path>>(mut self, file: P) -> Result<Self, Box<dyn Error>> {
        self.load_json(file)?;
        Ok(self)
    }

    pub fn with_load_toml<P: AsRef<Path>>(mut self, file: P) -> Result<Self, Box<dyn Error>> {
        self.load_toml(file)?;
        Ok(self)
    }

    pub fn get_line(&mut self, key: &str) -> Option<&str> {
        if let Some(lines) = self.line_pairs.get(key) {
            lines.iter().choose(&mut self.rng).map(|s| s.as_str())
        } else {
            None
        }
    }
}

impl Default for HotText<ThreadRng> {
    fn default() -> Self {
        Self::new(rand::thread_rng())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn load_file() {
        let mut ht = HotText::new(rand::thread_rng())
            .with_load_json("./test_lines.json")
            .unwrap();

        assert_eq!(
            ht.get_line("meta.welcome").unwrap(),
            "Welcome to the greatest dungeon crawler of all time!"
        );

        let mut ht = HotText::new(rand::thread_rng())
            .with_load_toml("./test_lines.toml")
            .unwrap();

        assert_eq!(
            ht.get_line("meta.welcome").unwrap(),
            "Welcome to the greatest dungeon crawler of all time!"
        );
    }

    #[test]
    fn insert() {
        let mut ht = HotText::new(rand::thread_rng());
        ht.insert(
            "meta.welcome",
            "Welcome to the greatest dungeon crawler of all time!",
        )
        .unwrap();

        assert_eq!(
            ht.get_line("meta.welcome").unwrap(),
            "Welcome to the greatest dungeon crawler of all time!"
        );
    }

    #[test]
    fn extend() {
        let mut ht = HotText::new(rand::thread_rng());
        ht.extend(
            "meta.welcome",
            vec!["Welcome to the greatest dungeon crawler of all time!".to_string()]
                .into_iter()
                .collect(),
        )
        .unwrap();

        assert_eq!(
            ht.get_line("meta.welcome").unwrap(),
            "Welcome to the greatest dungeon crawler of all time!"
        );
    }

    #[test]
    fn load_hashmap() {
        let mut hashmap: LinePairs = HashMap::new();
        hashmap.insert(
            "meta.welcome".to_string(),
            vec!["Welcome to the greatest dungeon crawler of all time!".to_string()]
                .into_iter()
                .collect(),
        );

        let mut ht = HotText::new(rand::thread_rng());
        ht.load_hashmap(hashmap).unwrap();

        assert_eq!(
            ht.get_line("meta.welcome").unwrap(),
            "Welcome to the greatest dungeon crawler of all time!"
        );
    }

    #[test]
    fn get_line() {
        let mut ht = HotText::new(rand::thread_rng())
            .with_load_json("./test_lines.json")
            .unwrap();
        assert_eq!(
            ht.get_line("meta.welcome").unwrap(),
            "Welcome to the greatest dungeon crawler of all time!"
        );
        assert!([
            "You encounter {enemy}!",
            "You stumble across {enemy}!",
            "Oh no! It's {enemy}!"
        ]
        .contains(&ht.get_line("combat.encounter").unwrap()));
    }
}