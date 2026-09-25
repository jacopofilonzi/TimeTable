/// `""`, `"/"` -> `""`; `"timetable/"` -> `"/timetable"`.
pub fn normalize(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("/{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn base_path_normalization() {
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("/"), "");
        assert_eq!(normalize("timetable"), "/timetable");
        assert_eq!(normalize("/timetable/"), "/timetable");
        assert_eq!(normalize("/a/b/"), "/a/b");
    }
}
