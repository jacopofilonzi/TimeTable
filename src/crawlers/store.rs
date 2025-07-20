use super::main::{ UniversityCrawler };
use super::unicam;



pub fn get_university_crawler(name: &str) -> Option<Box<dyn UniversityCrawler>> {
    match name {
        "unicam" => Some(Box::new(unicam::UnicamCrawler)),
        // Add other crawlers here
        _ => None,
    }
}