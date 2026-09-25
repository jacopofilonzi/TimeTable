//! Università di Camerino — <https://orarilezioni.unicam.it>
//!
//! Wizard ([`wizard`]): a single step with the course (remote, searchable) and the year (static
//! options).
//!
//! # Upstream
//!
//! - **Courses** ([`courses`]): HTML scraped from `https://orarilezioni.unicam.it/` (~680 KB).
//!   Courses are the options of `<select id="selectPercorsi">`, grouped by school:
//!   `<optgroup label="School"><option value="ID">CODE - NAME</option>…</optgroup>`.
//!   `value="0"` is a placeholder and is skipped.
//! - **Years**: not fetched; hard-coded in [`wizard`], copied from `<select id="selectAnno">` on the same
//!   page: `1`–`5`, plus `0` = all years.
//! - **Lessons** ([`lessons`]): JSON from `https://unifare.unicam.it/controller/ajaxController.php`
//!   with `filename=../didattica/controller/orari.php`, `class=OrariController`,
//!   `method=getDateLezioniByPercorsoCalendar`, `parametri[]=<course_id>`, `parametri[]=false`,
//!   `parametri[]=<year>`, `start`/`end` (RFC 3339, `end` exclusive). Returns
//!   `[{id, title, description, start, end, ...}]` where `start`/`end` are **epoch milliseconds**
//!   and `description` is HTML:
//!   `Location <div style="height:8px"></div><b>Docenti:</b> NAMES<br><i><b>Avvisi:</b> notes</i>`
//!   (teachers and notes are optional).
//!
//! # TODO
//!
//! Optimize: the courses page is heavy and regex-parsed. Look for a JSON endpoint on
//! `ajaxController.php` listing the study paths, like the lessons one.

mod courses;
mod html;
mod lessons;
mod wizard;

use async_trait::async_trait;

use self::wizard::{COURSE_ID, COURSE_YEAR};
use super::{DateRange, Params, University};
use crate::{
    errors::{AppError, NotFound},
    models::{Lesson, SelectOption, UniversityInfo},
};

pub struct Unicam {
    info: UniversityInfo,
}

impl Unicam {
    pub fn new() -> Self {
        Self {
            info: wizard::info(),
        }
    }
}

#[async_trait]
impl University for Unicam {
    fn info(&self) -> &UniversityInfo {
        &self.info
    }

    async fn options(
        &self,
        http: &reqwest::Client,
        field: &str,
        _params: &Params,
    ) -> Result<Vec<SelectOption>, AppError> {
        match field {
            COURSE_ID => courses::fetch(http).await,
            _ => Err(NotFound::new(format!("Field '{field}' has no remote options")).into()),
        }
    }

    async fn lessons(
        &self,
        http: &reqwest::Client,
        params: &Params,
        range: &DateRange,
    ) -> Result<Vec<Lesson>, AppError> {
        lessons::fetch(http, &params[COURSE_ID], &params[COURSE_YEAR], range).await
    }
}
