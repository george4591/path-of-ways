use super::model::Note;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Template {
    Blank,
    Build,
    Boss,
}

impl Template {
    pub fn label(self) -> &'static str {
        match self {
            Template::Blank => "Blank note",
            Template::Build => "Build planner",
            Template::Boss => "Boss strategy",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Template::Blank => "Empty note",
            Template::Build => "Class, ascendancy, gems, passives, PoB",
            Template::Boss => "Mechanics, loadout, strategy",
        }
    }

    pub fn body(self) -> &'static str {
        match self {
            Template::Blank => "",
            Template::Build => BUILD_TEMPLATE,
            Template::Boss => BOSS_TEMPLATE,
        }
    }

    pub fn tags(self) -> &'static [&'static str] {
        match self {
            Template::Blank => &[],
            Template::Build => &["build"],
            Template::Boss => &["boss"],
        }
    }

    pub fn apply(self, note: &mut Note) {
        let body = self.body();
        if !body.is_empty() {
            note.body = body.to_string();
        }
        for tag in self.tags() {
            let owned = (*tag).to_string();
            if !note.tags.contains(&owned) {
                note.tags.push(owned);
            }
        }
    }
}

const BUILD_TEMPLATE: &str = "## Class

## Ascendancy

## Skill setup
| Skill gem | Support gems |
|-----------|--------------|
|           |              |
|           |              |
|           |              |

## Key passives
-

## Path of Building
[paste link here]

## Notes
";

const BOSS_TEMPLATE: &str = "## Mechanics
-

## Loadout
- Flask:
- Movement:

## Strategy
1.

## Common deaths
-
";
