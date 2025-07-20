// External libraries
use chrono::{Datelike, Utc, Duration};
use std::collections::HashMap;
use async_trait::async_trait;
use reqwest::{Client};
use regex::Regex;

// Internal modules
use crate::models::{ lesson::Lesson, course::Course, error::{ Error, ErrorFault }};
use super::main::{ UniversityCrawler };



pub struct UnicamCrawler;

#[async_trait]
impl UniversityCrawler for UnicamCrawler 
{
    // ============================================================================================================ 

    async fn get_lessons(&self, query: HashMap<String, String>) -> Result<Vec<Lesson>, Error> {

        // Global parameters
        let mut to_return: Vec<Lesson> = vec![];
        let date_from: String;
        let date_to: String;
        let body: String;


        // User choosen parameters
        let course_id = match query.get("course_id") {
            Some(id) => id,
            None => return Err(Error {
                            error: "Bad request".into(),
                            http_code: Some(400),
                            message: Some("Missing course_id in query parameters".into()),
                            fault: ErrorFault::User
                        }),
        };

        let course_year = match query.get("course_year") {
            Some(year) => year,
            None => return Err(Error {
                            error: "Bad request".into(),
                            http_code: Some(400),
                            message: Some("Missing course_year in query parameters".into()),
                            fault: ErrorFault::User
                        }),
        };


        let weeks: i8 = query        
                    .get("weeks")
                    .and_then(|s| s.parse::<i8>().ok()) // try to parse
                    .unwrap_or(3); // default to 3 weeks if missing or invalid;

                    
        //-----------------------------------------------------------------------------------


        {//Parse the input fields

            // Valid year from 0 to 5
            if !Regex::new(r"^[0-5]$").unwrap().is_match(course_year) {
                return Err(Error {
                    error: "Bad request".into(),
                    http_code: Some(400),
                    message: Some("course_year must be a number from 0 to 5".into()),
                    fault: ErrorFault::User
                });
            }

            // Validate weeks from 1 to 5
            if weeks < 1 || weeks > 5 {
                return Err(Error {
                    error: "Bad request".into(),
                    http_code: Some(400),
                    message: Some("Weeks must be between 1 and 5".into()),
                    fault: ErrorFault::User
                });
            }

        }

        //-----------------------------------------------------------------------------------

        {//Set the time span

            let today = Utc::now(); // Just date, no time

            // Get the monday of the current week
            let weekday = today.weekday();
            let day_since_monday = weekday.num_days_from_monday() as i64;
            let start = today - Duration::days(day_since_monday);

            // Get the end of the specified period
            let end = start + Duration::weeks(weeks as i64) - Duration::days(1);

            date_from = start.to_rfc3339();
            date_to = end.to_rfc3339();

            // println!("Date from: {}\nDate to: {}", date_from, date_to);
        }

        //-----------------------------------------------------------------------------------

        {// Request maker
            let _request = Client::new()
                .get("https://unifare.unicam.it//controller/ajaxController.php")
                .query(&[
                    ("filename", "../didattica/controller/orari.php"),
                    ("class", "OrariController"),
                    ("method", "getDateLezioniByPercorsoCalendar"),
                    ("parametri[]", course_id),
                    ("parametri[]", "false"),
                    ("parametri[]", course_year),
                    ("start", date_from.as_str()),
                    ("end", date_to.as_str()),
                    // ("start", "2025-06-02T14:44:09.523442800+00:00"),   // For testing purposes
                    // ("end", "2025-06-15T14:44:09.523442800+00:00")      // For testing purposes
                ]);

            let _response = _request.send().await;
            
            
            // Check if the request was successful
            match _response {
                Ok(_response_unwrapped) => {

                    let _status_code = _response_unwrapped.status();
                    if  _status_code != 200 {
                        return Err(Error {
                            error: "Error while crawling lessons from unicam".into(),
                            http_code: None,
                            message: Some(format!("The server responded with status code: {}", _status_code)),
                            fault: ErrorFault::External
                        });
                    }

                    // Parse the response
                    // println!("{:#?}", _result.ok().unwrap().text());

                    body = _response_unwrapped.text().await.unwrap();
                }

                Err(err) => {
                    return Err(Error {
                        error: "Error while crawling lessons from unicam".into(),
                        http_code: None,
                        message: Some(format!("Connection error while attempting to fetch: {}", err)),
                        fault: ErrorFault::Internal
                    });
                }
            }
        }

        //-----------------------------------------------------------------------------------

        {//Parsing body

            // Parse the JSON response
            let _json = serde_json::from_str::<serde_json::Value>(&body)
                .map_err(|error| Error {
                    error: "Error while parsing crawled data from unicam".into(),
                    http_code: None,
                    message: Some(format!("Parsing error: {:#?} \nRequest query: {:#?}\nFrom: {:#?}\nTo: {:#?} \nBody: {:#?}",error, query, date_from, date_to, body )),
                    fault: ErrorFault::Internal
                })?;

            // println!("JSON: {:#?}", _json);

            if let Some(lessons) = _json.as_array() {
                for lesson in lessons {

                    let description_string = lesson["description"].to_string();
                    let description_split = description_string.split(" <div style=\\\"height:8px\\\"></div><b>Docenti:</b> ").collect::<Vec<&str>>();
                    
                    to_return.push(Lesson {
                        starts_at: lesson["start"].to_string(),
                        ends_at: lesson["end"].to_string(),
                        subject: lesson["title"].as_str().unwrap_or("").to_string(),
                        location: Some(description_split[0].to_string()),
                        teacher: Some(description_split[1].to_string()),
                        description: None
                    })
                }
            } else {
                Error {
                    error: "Error while parsing crawled data from unicam".into(),
                    http_code: None,
                    message: Some(format!("JSON response is not an array \nRequest query: {:#?}\nFrom: {:#?}\nTo: {:#?} \nJson data: {:#?}", query, date_from, date_to, _json)),
                    fault: ErrorFault::Internal
                };
            }

        }
        
        
        Ok(to_return)

    }

    // ============================================================================================================
    
    async fn get_courses(&self, _query: HashMap<String, String>) ->  Result<Vec<Course>, Error> {

        // Global parameters
        let mut to_return: Vec<Course> = vec![];
        let _html: String;

        {//Request maker

            let _request = Client::new()
                    .get("https://orarilezioni.unicam.it/");
            
            let _response = _request.send().await;
    
            match _response {
                Ok(_response_unwrapped) => {
    
                    let _status_code = _response_unwrapped.status();
                    if  _status_code != 200 {
                        return Err(Error {
                            error: "Error while crawling courses from unicam".into(),
                            http_code: None,
                            message: Some(format!("The server responded with status code: {}", _status_code)),
                            fault: ErrorFault::External
                        });
                    }
    
                    // Parse the response
                    _html = _response_unwrapped.text().await.unwrap();

                }
                Err(err) => {
                    return Err(Error {
                        error: "Error while crawling data from unicam".into(),
                        http_code: None,
                        message: Some(format!("Connection error while attempting to fetch: {}", err)),
                        fault: ErrorFault::Internal
                    });
                }
            }
        }

        //-----------------------------------------------------------------------------------

        {    
            // Regex per catturare optgroup e il suo contenuto
            let optgroup_regex = Regex::new(r#"<optgroup\s+label="([^"]+)"[^>]*>([\s\S]*?)</optgroup>"#).unwrap();
            // Regex per catturare le option all'interno di un optgroup
            let option_regex = Regex::new(r#"<option(?:\s+[^>]*)?\s+value="(\d+)"[^>]*>([^<]+)</option>"#).unwrap();
    

            // Iterating over optgroups in the HTML
            for optgroup_captures in optgroup_regex.captures_iter(_html.as_str()) {
                let category = optgroup_captures.get(1).unwrap().as_str();
                let optgroup_content = optgroup_captures.get(2).unwrap().as_str();

                // println!("\nOptgroup: {}", category);
                // println!("Content: {:#?}", optgroup_content);

                // Check if the category is empty
                if optgroup_content.trim().is_empty() {continue};


                // Iterating over options within the optgroup
                for option_captures in option_regex.captures_iter(optgroup_content) {
                    let course_id = option_captures.get(1).unwrap().as_str();
                    let opt_inner_split = option_captures.get(2).unwrap().as_str().trim().split(" - ").collect::<Vec<&str>>();
                    let course_code = opt_inner_split.get(0).unwrap_or(&"").to_string();
                    let course_name = opt_inner_split.get(1).unwrap_or(&"").to_string();

                    // println!("\tOption:\t{}\t{}\t{}", course_id, course_code, course_name);


                    // Add the course to the return vector
                    to_return.push(Course {
                        category: category.to_string(),
                        id: course_id.to_string(),
                        code: course_code,
                        name: course_name,
                    })

                }
            }

        }


        Ok(to_return)
    }

    // ============================================================================================================

}
