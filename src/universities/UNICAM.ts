import { JSDOM }  from "jsdom";
import Course from "../types/Course";
import Capitalize from "../utilities/capitalize";
import Lesson from "../types/Lesson";
import University from "../types/University";
import UniversityError from "../types/Errors";

export default new University({
    name: "Università di Camerino",
    denomination: "UNICAM",
    //---------------------------------------------------------------------------------------------------
    getCourses: async () => {

        //Fetch the page
        const response = await fetch("https://orarilezioni.unicam.it/")

        //Check if the page is valid
        if (!response.ok) {
            throw new UniversityError({userFault: false, message: `Error fetching data: ${response.statusText}`});
        }

        const html = await response.text()
        const dom = new JSDOM(html)

        const courses: Course[] = []

        if (!dom.window.document.getElementById("selectPercorsi")) {
            throw new UniversityError({userFault: false, message: "selectPercorsi element not found"});
        }


        for (const group of dom.window.document.getElementById("selectPercorsi")?.children || []) {
            for (const coruse_raw of group.children || []) {
                courses.push({
                    id: coruse_raw.getAttribute("value")!,
                    code: coruse_raw.innerHTML.split(" - ")[0],
                    name: Capitalize(coruse_raw.innerHTML.split(" - ")[1].toLocaleLowerCase()),
                    category: group.getAttribute("label") || undefined
                })
            }
        }

        return courses;
    },
    //---------------------------------------------------------------------------------------------------
    getLessons: async (query: any) => {

        if (!query["course_id"]) {
            throw new UniversityError({userFault: true, message: "Missing course_id as query parameter."});
        }

        if (!query["course_year"]) {
            throw new UniversityError({userFault: true, message: "Missing course_year as query parameter."});
        }

        // Validate course_year as number -> 0 < X < 5
        if (!/^[0-5]$/.test(query.course_year)) {
            throw new UniversityError({userFault: true, message: "Invalid course_year. It must be a number between 0 and 5."});
        }

        // Set the starting date to the beginning of the week
        const startDate = new Date();
        startDate.setUTCHours(0, 0, 0, 0);
        startDate.setUTCDate(startDate.getUTCDate() - startDate.getUTCDay());
    
        // Set the ending date to the end of the requested week to 3 weeks ahead
        const endDate = new Date(startDate);
        endDate.setUTCDate(startDate.getUTCDate() + (7 * 3));


        //Build request

        const url = new URL("https://unifare.unicam.it//controller/ajaxController.php");
            url.searchParams.append("filename", "../didattica/controller/orari.php");       //
            url.searchParams.append("class", "OrariController");                            //
            url.searchParams.append("method", "getDateLezioniByPercorsoCalendar");          //
            url.searchParams.append("parametri[]", query.course_id);                              // Course ID
            url.searchParams.append("parametri[]", "false");                                //
            url.searchParams.append("parametri[]", query.course_year);                            // Course Year
            url.searchParams.append("start", startDate.toISOString());                      // Start date
            url.searchParams.append("end", endDate.toISOString());                          // End date


        const response = await fetch(url.toString());

        if (!response.ok) {
            throw new UniversityError({userFault: false, message: `Error fetching data: ${response.statusText}`});
        }
    
        const lezioni_raw = await response.json();
        const lessons: Lesson[] = [];

        for (const element of lezioni_raw) {
            lessons.push({
                starts_at: new Date(element.start),
                ends_at: new Date(element.end),
                subject: element.title,
                Teacher: element.description.split(` <div style="height:8px"></div><b>Docenti:</b> `)[1],
                location: element.description.split(` <div style="height:8px"></div><b>Docenti:</b> `)[0]
            })
        }


        return lessons;
    }
})