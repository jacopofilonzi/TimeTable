//! Client family from the `User-Agent`, for bounded metric labels and quick filtering.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Client {
    Google,
    Apple,
    Outlook,
    Thunderbird,
    /// ICSx⁵ / DAVx⁵ on Android.
    Icsx5,
    Curl,
    Python,
    Browser,
    Other,
}

impl Client {
    pub fn from_user_agent(user_agent: Option<&str>) -> Self {
        let Some(ua) = user_agent.map(str::to_ascii_lowercase) else {
            return Self::Other;
        };
        let has = |s: &str| ua.contains(s);
        // Order matters: calendar apps often also say `Mozilla/`.
        if has("google-calendar") || (has("google") && has("calendar")) {
            Self::Google
        } else if has("dataaccessd") || has("calendaragent") || has("calendarstore") {
            Self::Apple
        } else if has("outlook") || has("exchange") || has("microsoft office") {
            Self::Outlook
        } else if has("thunderbird") {
            Self::Thunderbird
        } else if has("icsx5") || has("davx5") {
            Self::Icsx5
        } else if ua.starts_with("curl/") {
            Self::Curl
        } else if has("python") {
            Self::Python
        } else if ua.starts_with("mozilla/") {
            Self::Browser
        } else {
            Self::Other
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Apple => "apple",
            Self::Outlook => "outlook",
            Self::Thunderbird => "thunderbird",
            Self::Icsx5 => "icsx5",
            Self::Curl => "curl",
            Self::Python => "python",
            Self::Browser => "browser",
            Self::Other => "other",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Client;

    fn classify(ua: &str) -> Client {
        Client::from_user_agent(Some(ua))
    }

    #[test]
    fn classifies_user_agents() {
        assert_eq!(classify("Google-Calendar-Importer"), Client::Google);
        assert_eq!(
            classify("iOS/17.5.1 (21F90) dataaccessd/1.0"),
            Client::Apple
        );
        assert_eq!(
            classify("macOS/14.5 (23F79) CalendarAgent/988.5.4.1"),
            Client::Apple
        );
        assert_eq!(
            classify("Microsoft Office/16.0 (Windows NT 10.0; Microsoft Outlook 16.0.17928; Pro)"),
            Client::Outlook
        );
        assert_eq!(
            classify(
                "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Thunderbird/128.3.0"
            ),
            Client::Thunderbird
        );
        assert_eq!(
            classify("ICSx5/2.2.3 (ical4j/3.2.19; okhttp/4.12.0; Android 14)"),
            Client::Icsx5
        );
        assert_eq!(classify("curl/8.9.1"), Client::Curl);
        assert_eq!(classify("python-requests/2.32.3"), Client::Python);
        assert_eq!(
            classify(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/129.0 Safari/537.36"
            ),
            Client::Browser
        );
        assert_eq!(classify("Go-http-client/2.0"), Client::Other);
        assert_eq!(Client::from_user_agent(None), Client::Other);
    }
}
