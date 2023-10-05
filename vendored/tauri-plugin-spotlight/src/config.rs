use std::collections::HashMap;

#[derive(serde::Deserialize, Default, Debug, Clone, PartialEq)]
pub struct WindowConfig {
    pub label: String,
    pub shortcut: String,
    pub macos_window_level: Option<i32>,
}

#[derive(serde::Deserialize, Default, Debug, Clone, PartialEq)]
pub struct PluginConfig {
    pub windows: Option<Vec<WindowConfig>>,
    pub global_close_shortcut: Option<String>,
}

impl PluginConfig {
    pub fn merge(a: &Self, b: &Self) -> Self {
        let mut windows: Vec<WindowConfig> = vec![];
        if let Some(w) = a.windows.clone() {
            windows = w;
        } else if let Some(w) = b.windows.clone() {
            windows = w;
        }
        let mut dict: HashMap<String, String> = HashMap::default();
        for w in &windows {
            dict.insert(w.label.clone(), w.shortcut.clone());
        }
        if let Some(w) = b.windows.clone() {
            for config in w {
                if !dict.contains_key(&config.label) {
                    windows.push(WindowConfig {
                        label: config.label,
                        shortcut: config.shortcut,
                        macos_window_level: config.macos_window_level,
                    });
                }
            }
        }
        Self {
            windows: {
                if windows.len() == 0 {
                    None
                } else {
                    Some(windows)
                }
            },
            global_close_shortcut: a.global_close_shortcut.clone().or(b.global_close_shortcut.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WindowConfig;
    use super::PluginConfig;

    #[test]
    fn merge_and_override_default_value() {
        let a = PluginConfig::default();
        let b = PluginConfig {
            windows: Some(vec![
                WindowConfig {
                    label: String::from("main"),
                    shortcut: String::from("Ctrl+I"),
                    macos_window_level: None,
                },
            ]),
            global_close_shortcut: Some(String::from("Escape")),
        };
        let c = PluginConfig::merge(&a, &b);
        assert_eq!(c, b);
    }

    #[test]
    fn merge_windows() {
        let a = PluginConfig {
            windows: Some(vec![
                WindowConfig {
                    label: String::from("main"),
                    shortcut: String::from("Ctrl+I"),
                    macos_window_level: None,
                },
            ]),
            global_close_shortcut: None,
        };
        let b = PluginConfig {
            windows: Some(vec![
                WindowConfig {
                    label: String::from("foo"),
                    shortcut: String::from("bar"),
                    macos_window_level: None,
                },
            ]),
            global_close_shortcut: None,
        };
        let c = PluginConfig::merge(&a, &b);
        assert_eq!(c, PluginConfig {
            windows: Some(vec![
                WindowConfig {
                    label: String::from("main"),
                    shortcut: String::from("Ctrl+I"),
                    macos_window_level: None,
                },
                WindowConfig {
                    label: String::from("foo"),
                    shortcut: String::from("bar"),
                    macos_window_level: None,
                },
            ]),
            global_close_shortcut: None,
        });
    }

    #[test]
    fn a_takes_precedence_over_b() {
        let a = PluginConfig {
            windows: None,
            global_close_shortcut: Some(String::from("Escape")),
        };
        let b = PluginConfig {
            windows: None,
            global_close_shortcut: Some(String::from("baz")),
        };
        let c = PluginConfig::merge(&a, &b);
        assert_eq!(c, a);
    }
}
