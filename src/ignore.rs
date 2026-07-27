use std::path::Path;

use crate::constants;

#[derive(Debug)]
pub struct ExcludeRule {
    pub pattern: String,
    pub negated: bool,
    pub directory_only: bool,
    pub anchored: bool,
}

pub struct ExcludeRules {
    pub patterns: Vec<ExcludeRule>,
}

impl ExcludeRules {
    pub fn new() -> Self {
        let mut rules = Self { patterns: Vec::new() };
        rules.set_defaults();
        rules
    }

    pub fn load_or_default(exclude_path: Option<&std::path::Path>, settings_patterns: &[String]) -> Self {
        let rules = if let Some(p) = exclude_path {
            if p.exists() {
                Self::load(p)
            } else {
                let mut r = Self::new();
                r.set_patterns_from_strings(&settings_patterns);
                r
            }
        } else {
            Self::load_or_from_docpackignore(settings_patterns)
        };
        rules
    }

    pub fn load_or_from_docpackignore(settings_patterns: &[String]) -> Self {
        if Path::new(".docpackignore").exists() {
            return Self::load(Path::new(".docpackignore"));
        }
        let mut r = Self::new();
        r.set_patterns_from_strings(&settings_patterns);
        r
    }

    pub fn from_rules_text(rules: &str) -> Self {
        let mut r = Self { patterns: Vec::new() };
        for line in rules.lines() {
            if let Some(rule) = parse_gitignore_line(line) {
                r.patterns.push(rule);
            }
        }
        r
    }

    fn set_defaults(&mut self) {
        for p in Self::default_patterns() {
            let rule = ExcludeRule {
                pattern: p.to_string(),
                negated: false,
                directory_only: p.ends_with('/'),
                anchored: false,
            };
            self.patterns.push(rule);
        }
    }

    fn set_patterns_from_strings(&mut self, patterns: &[String]) {
        for p in patterns {
            let rule = ExcludeRule {
                pattern: p.clone(),
                negated: false,
                directory_only: p.ends_with('/'),
                anchored: false,
            };
            self.patterns.push(rule);
        }
    }

    pub fn default_patterns() -> &'static [&'static str] {
        constants::DEFAULT_EXCLUDES
    }

    pub fn load(path: &Path) -> Self {
        let mut rules = Self::new();
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if let Some(rule) = parse_gitignore_line(line) {
                    rules.patterns.push(rule);
                }
            }
        }
        rules
    }

    pub fn add_pattern(&mut self, pattern: &str) {
        let negated = pattern.starts_with('!');
        let dir_only = pattern.ends_with('/');
        let anchored = pattern.starts_with('/') && !pattern.starts_with("//");
        let p = if negated { &pattern[1..] } else { pattern };
        let p = if dir_only { &p[..p.len()-1] } else { p };
        let rule = ExcludeRule {
            pattern: unescape_gitignore(p),
            negated,
            directory_only: dir_only,
            anchored,
        };
        self.patterns.push(rule);
    }

    pub fn patterns(&self) -> Vec<String> {
        self.patterns.iter().map(|r| r.pattern.clone()).collect()
    }

    pub fn is_excluded(&self, rel_path: &str) -> bool {
        let path = rel_path.replace('\\', "/");
        let mut any_positive = false;
        let mut last_was_negated = false;

        for rule in &self.patterns {
            let m = glob_match(&rule.pattern, &path, rule.directory_only, rule.anchored);
            if m {
                if rule.negated {
                    // Negation only takes effect if a prior positive matched this file
                    if any_positive {
                        last_was_negated = true;
                    }
                } else {
                    any_positive = true;
                    last_was_negated = false;
                }
            }
        }

        if !any_positive {
            return false;
        }

        !last_was_negated
    }
}

impl Default for ExcludeRules {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_gitignore_line(line: &str) -> Option<ExcludeRule> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    // Strip comments (# at start)
    if line.starts_with('#') {
        return None;
    }

    let mut negated = false;
    let mut s = line;
    if s.starts_with('!') {
        negated = true;
        s = &s[1..];
    }

    let mut directory_only = false;
    let mut anchored = false;

    if s.ends_with('/') {
        directory_only = true;
        s = &s[..s.len() - 1];
    }

    if s.starts_with('/') && !s.starts_with("//") {
        anchored = true;
    }

    // Handle escaped characters
    let pattern = unescape_gitignore(s);

    Some(ExcludeRule {
        pattern,
        negated,
        directory_only,
        anchored,
    })
}

fn unescape_gitignore(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            result.push(chars[i + 1]);
            i += 2;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

fn glob_match(pattern: &str, path: &str, directory_only: bool, anchored: bool) -> bool {
    if pattern.is_empty() {
        return false;
    }

    // Pattern with / is automatically anchored
    let is_anchored = anchored || pattern.contains('/');

    if is_anchored {
        let pat = if pattern.starts_with('/') { &pattern[1..] } else { &pattern };
        if directory_only {
            return path == pat || path.starts_with(&format!("{}/", pat));
        }
        return glob_match_inner(pat, &path);
    }

    // Non-anchored: basename matching against each component
    if directory_only {
        // Match if any path component matches the pattern
        return path.split('/').any(|component| glob_match_inner(&pattern, component));
    }

    // Full path match or basename match
    if glob_match_inner(&pattern, &path) {
        return true;
    }

    path.split('/').any(|component| glob_match_inner(&pattern, component))
}

fn glob_match_inner(pattern: &str, path: &str) -> bool {
    let p_chars: Vec<char> = pattern.chars().collect();
    let s_chars: Vec<char> = path.chars().collect();
    glob_recursive(&p_chars, &s_chars, 0, 0)
}

fn glob_recursive(p: &[char], s: &[char], pi: usize, si: usize) -> bool {
    if pi == p.len() {
        return si == s.len();
    }

    match p[pi] {
        '*' => {
            if pi + 1 < p.len() && p[pi + 1] == '*' {
                let after_stars = pi + 2;
                if after_stars >= p.len() {
                    return true;
                }
                if p[after_stars] == '/' {
                    let after_slash = after_stars + 1;
                    if glob_recursive(p, s, after_slash, si) {
                        return true;
                    }
                    let mut j = si;
                    while j < s.len() {
                        if s[j] == '/' {
                            if glob_recursive(p, s, pi, j + 1) {
                                return true;
                            }
                        }
                        j += 1;
                    }
                    return false;
                }
            }
            if glob_recursive(p, s, pi + 1, si) {
                return true;
            }
            if si < s.len() && s[si] != '/' {
                glob_recursive(p, s, pi, si + 1)
            } else {
                false
            }
        }
        '?' => {
            if si < s.len() && s[si] != '/' {
                glob_recursive(p, s, pi + 1, si + 1)
            } else {
                false
            }
        }
        c => {
            if si < s.len() && s[si] == c {
                glob_recursive(p, s, pi + 1, si + 1)
            } else {
                false
            }
        }
    }
}
