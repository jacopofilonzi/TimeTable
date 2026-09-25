//! Data exposed by the API: lessons and the per-university wizard schema.

mod field;
mod lesson;
mod select_option;
mod step;
mod text;
mod university_info;

pub use field::{Field, FieldKind};
pub use lesson::Lesson;
pub use select_option::SelectOption;
pub use step::Step;
pub use text::Text;
pub use university_info::{UniversityInfo, WeeksRange};
