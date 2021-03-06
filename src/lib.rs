use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

use rand::prelude::*;

// TODO: Only evaluate values that are used in formatting the line chosen

/// Gets one line with the specified key from HotText.
///
/// # Panics
/// Panics if the key does not exist.
#[macro_export]
macro_rules! get_line {
    ($ht:expr, $k:expr) => {
        $ht.get_line_raw($k)
            .expect(&format!("No line with key {}", $k))
    };
}

/// Gets all lines with the specified key from HotText and returns them as a vector.
///
/// # Panics
/// Panics if the key does not exist.
#[macro_export]
macro_rules! get_lines {
    ($ht:expr, $k:expr) => {
        $ht.get_lines_raw($k)
            .expect(&format!("No lines with key {}", $k))
    };
}

/// Gets one line with the specified key from HotText and formats it using the provided arguments.
///
/// # Panics
/// Panics if the key does not exist.
#[macro_export]
macro_rules! fmt_line {
    ($ht:expr, $k:expr, $($key:ident = $value:expr),+) => {
        $ht.render_line(
            $k,
            vec![$((stringify!($key), $value)),+]
        ).expect("Failed to find or format line.")
    };
}

type LinePairs = HashMap<String, HashSet<String>>;

#[derive(Debug)]
pub struct TemplateCompileError {}

impl fmt::Display for TemplateCompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TemplateCompileError")
    }
}

impl std::error::Error for TemplateCompileError {}

/// Used to store, retrieve, and format HotText template lines.
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

    /// Insert one key/line pair into the collection.
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

    /// Insert multiple lines with a shared key into the collection.
    pub fn extend(&mut self, key: &str, new_lines: HashSet<String>) -> Result<(), Box<dyn Error>> {
        if let Some(lines) = self.line_pairs.get_mut(key) {
            lines.extend(new_lines);
        } else {
            self.line_pairs.insert(key.to_string(), new_lines);
        }
        Ok(())
    }

    /// Insert multiple key/line pairs into the collection.
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

    /// Loads key/line pairs from a .json file
    pub fn load_json<P: AsRef<Path>>(&mut self, file: P) -> Result<(), Box<dyn Error>> {
        let file = fs::File::open(file)?;
        let line_pairs: LinePairs = serde_json::from_reader(file)?;
        self.load_hashmap(line_pairs)
    }

    /// Loads key/line pairs from a .toml file
    pub fn load_toml<P: AsRef<Path>>(&mut self, file: P) -> Result<(), Box<dyn Error>> {
        let content = fs::read_to_string(file)?;
        let line_pairs: LinePairs = toml::from_str(&content)?;
        self.load_hashmap(line_pairs)
    }

    /// Chainable variant of [`HotText::load_json()`]
    pub fn with_load_json<P: AsRef<Path>>(mut self, file: P) -> Result<Self, Box<dyn Error>> {
        self.load_json(file)?;
        Ok(self)
    }

    /// Chainable variant of [`HotText::load_toml()`]
    pub fn with_load_toml<P: AsRef<Path>>(mut self, file: P) -> Result<Self, Box<dyn Error>> {
        self.load_toml(file)?;
        Ok(self)
    }

    /// Gets one line with the specified key as a [`String`].
    pub fn get_line_raw(&mut self, key: &str) -> Option<String> {
        if let Some(lines) = self.line_pairs.get(key) {
            lines.iter().choose(&mut self.rng).cloned()
        } else {
            None
        }
    }

    /// Gets all lines with the specified key as [`String`]s.
    pub fn get_lines_raw(&mut self, key: &str) -> Option<HashSet<String>> {
        if let Some(lines) = self.line_pairs.get(key) {
            Some(lines.clone())
        } else {
            None
        }
    }

    /// Gets one line with the specified key compiled as a [`mustache::Template`].
    pub fn get_line(&mut self, key: &str) -> Result<mustache::Template, Box<dyn Error>> {
        let raw_line = self.get_line_raw(key).ok_or(TemplateCompileError {})?;
        Ok(mustache::compile_str(&raw_line)?)
    }

    /// Gets one line with the specified key and formats it using the provided data.
    pub fn render_line<'a, D: IntoIterator<Item = (&'a str, &'a str)>>(
        &mut self,
        key: &str,
        data: D,
    ) -> Result<String, Box<dyn Error>> {
        let raw_line = self.get_line_raw(key).ok_or(TemplateCompileError {})?;
        let template = mustache::compile_str(&raw_line)?;
        let data: HashMap<&str, &str> = data.into_iter().collect();
        Ok(template.render_to_string(&data)?)
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
            ht.get_line_raw("meta.welcome").unwrap(),
            "Welcome to the greatest dungeon crawler of all time!"
        );

        let mut ht = HotText::new(rand::thread_rng())
            .with_load_toml("./test_lines.toml")
            .unwrap();

        assert_eq!(
            ht.get_line_raw("meta.welcome").unwrap(),
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
            ht.get_line_raw("meta.welcome").unwrap(),
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
            ht.get_line_raw("meta.welcome").unwrap(),
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
            ht.get_line_raw("meta.welcome").unwrap(),
            "Welcome to the greatest dungeon crawler of all time!"
        );
    }

    #[test]
    fn get_line_raw() {
        let mut ht = HotText::new(rand::thread_rng())
            .with_load_json("./test_lines.json")
            .unwrap();
        assert_eq!(
            ht.get_line_raw("meta.welcome").unwrap(),
            "Welcome to the greatest dungeon crawler of all time!"
        );
        assert!([
            "You encounter a lion!",
            "You stumble across a tiger!",
            "Oh no! It's a bear!",
            "Oh my, it's a dragon!"
        ]
        .contains(&ht.get_line_raw("combat.encounter").unwrap().as_str()));
    }

    #[test]
    fn format_line() {
        let mut ht = HotText::new(rand::thread_rng());
        ht.insert("killed", "You were killed by {{{enemy}}}.")
            .unwrap();

        assert_eq!(
            ht.render_line("killed", vec![("enemy", "your mom")])
                .unwrap(),
            "You were killed by your mom."
        );
    }

    #[test]
    fn macros_get_line() {
        let mut ht = HotText::new(rand::thread_rng());
        ht.insert("killed", "You were killed by a meteorite.")
            .unwrap();

        assert_eq!(get_line!(ht, "killed"), "You were killed by a meteorite.");
    }

    #[test]
    fn macros_get_lines() {
        let mut ht = HotText::new(rand::thread_rng());
        ht.insert("killed", "You were killed by a meteorite.")
            .unwrap();
        ht.insert("killed", "A zombie ate your face.").unwrap();

        assert!(get_lines!(ht, "killed").into_iter().eq(vec![
            "You were killed by a meteorite.",
            "A zombie ate your face."
        ]
        .into_iter()));
    }

    #[test]
    fn macros_fmt_line() {
        let mut ht = HotText::new(rand::thread_rng());
        ht.insert("enter.first-person", "You enter {{{location}}}.")
            .unwrap();
        ht.insert("enter.third-person", "{{{name}}} enters {{{location}}}.")
            .unwrap();

        assert_eq!(
            fmt_line!(
                ht,
                "enter.first-person",
                location = "Dino Dan's Tunnel Network"
            ),
            "You enter Dino Dan's Tunnel Network."
        );
        assert_eq!(
            fmt_line!(
                ht,
                "enter.third-person",
                name = "Neo",
                location = "The Matrix"
            ),
            "Neo enters The Matrix."
        );
    }
}
