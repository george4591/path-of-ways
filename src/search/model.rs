/// One row in the quick switcher. Each variant is shaped just enough to
/// render a (primary, secondary) text pair and to dispatch a navigation
/// action when picked.
#[derive(Clone)]
pub enum Result {
    Note {
        id: String,
        title: String,
    },
    Zone {
        name: String,
        act_name: String,
        zone_id: String,
    },
    Boss {
        name: String,
        zone_name: String,
        act_name: String,
    },
    Recipe {
        name: String,
        category: String,
    },
    Link {
        url: String,
        title: String,
        domain: String,
    },
}

/// Labels used when rendering a result row: `(primary, secondary)`.
pub fn render_labels(result: &Result) -> (String, String) {
    match result {
        Result::Note { title, .. } => (title.clone(), String::new()),
        Result::Zone { name, act_name, .. } => (name.clone(), act_name.clone()),
        Result::Boss {
            name,
            zone_name,
            act_name,
            ..
        } => (name.clone(), format!("{} · {}", zone_name, act_name)),
        Result::Recipe { name, category } => (name.clone(), category.clone()),
        Result::Link { title, domain, .. } => (title.clone(), domain.clone()),
    }
}
