//! Unicam's wizard schema: a single step with the course (remote, searchable) and the year
//! (static options).

use crate::models::{Field, FieldKind, SelectOption, Step, Text, UniversityInfo, WeeksRange};

/// Query parameter of the course field; its options come from [`super::courses`].
pub const COURSE_ID: &str = "course_id";
/// Query parameter of the year field.
pub const COURSE_YEAR: &str = "course_year";

/// `(value, it, en)` for the year field, copied from `<select id="selectAnno">`; `0` = all years.
const YEARS: [(&str, &str, &str); 6] = [
    ("1", "Primo anno", "First year"),
    ("2", "Secondo anno", "Second year"),
    ("3", "Terzo anno", "Third year"),
    ("4", "Quarto anno", "Fourth year"),
    ("5", "Quinto anno", "Fifth year"),
    ("0", "Tutti gli anni", "All years"),
];

pub fn info() -> UniversityInfo {
    UniversityInfo {
        id: "unicam",
        name: Text::localized("Università di Camerino", "University of Camerino"),
        website: Some("https://orarilezioni.unicam.it"),
        steps: vec![course_step()],
        weeks: WeeksRange {
            min: 1,
            max: 5,
            default: 4,
        },
    }
}

/// Course and year on the same screen.
fn course_step() -> Step {
    Step {
        id: "course",
        title: Text::localized("Corso di studio", "Degree course"),
        description: Some(Text::localized(
            "Scegli il tuo corso di laurea e l'anno che frequenti.",
            "Pick your degree course and the year you are attending.",
        )),
        fields: vec![
            Field {
                key: COURSE_ID,
                label: Text::localized("Corso", "Course"),
                kind: FieldKind::RemoteSelect { searchable: true },
                depends_on: &[],
                pattern: Some(r"^\d{1,10}$"),
            },
            Field {
                key: COURSE_YEAR,
                label: Text::localized("Anno", "Year"),
                kind: FieldKind::Select {
                    options: year_options(),
                    default: Some("1"),
                },
                depends_on: &[],
                pattern: None,
            },
        ],
    }
}

fn year_options() -> Vec<SelectOption> {
    YEARS
        .into_iter()
        .map(|(value, it, en)| SelectOption {
            value: value.into(),
            label: Text::localized(it, en),
            group: None,
            hint: None,
        })
        .collect()
}
