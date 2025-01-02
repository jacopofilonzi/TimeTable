import express from 'express';
import { universities } from '../store';
import UniversityError from '../types/Errors';
import ParseDate from '../utilities/parseDate';
import ical from 'ical-generator';

const router = express.Router();

//#region Timetable routes

//Get Universities in JSON
router.get("/universities", (req, res) => {

    const toSend: { [denomination: string]: string } = {};

    for (const uni of universities) {
        toSend[uni[0]] = uni[1].Name;
    }
    
    res.send(toSend);
});


//Get courses of a university in JSON
router.get("/:university/courses", async (req, res) => {
    const universita = universities.get(req.params.university);

    if (!universita) {
        res.status(404).send({error: 404, message: "Università non trovata"});
        return;
    }

    try {
        // const courses = await universita.GetCourses(req.query);
        const courses = await universita.CacheCourses();
        res.send(courses);
    } catch (error: any) {

        if (error instanceof UniversityError && error.UserFault) {
            res.status(400).send({error: 400, message: error.message});
            return;
        }

        console.error(`-------------------------------------------------`);
        console.error(`[TimeTable] Error => Where: ${req.url}`);
        console.error(`[TimeTable] Error => When: ${ParseDate(new Date)}`);
        console.error(`[TimeTable] Error => \`${error}\``);
        console.error(`-------------------------------------------------`);

        res.status(500).send({error: 500, message: "Server error"});
    }    
    
});

//---------------------------------------------------------------------------------------------

//Get course's lessons of a university in JSON
router.get("/:university/lessons", async (req, res) => {
    const universita = universities.get(req.params.university);

    if (!universita) {
        res.status(404).send({error: 404, message: "Università non trovata"});
        return;
    }

    try {
        const lessons = await universita.CacheLessons((req.query.course_id as string), (req.query.course_year as string));
        res.send(lessons);
    } catch (error: any) {

        if (error instanceof UniversityError && error.UserFault) {
            res.status(400).send({error: 400, message: error.message});
            return;
        }

        console.error(`-------------------------------------------------`);
        console.error(`[TimeTable] Error => Where: ${req.url}`);
        console.error(`[TimeTable] Error => ${ParseDate(new Date)}`);
        console.error(`[TimeTable] Error => \`${error}\``);
        console.error(`-------------------------------------------------`);

        res.status(500).send({error: 500, message: "Server error"});

        res.send()
    }
})

//---------------------------------------------------------------------------------------------

//Get course's lessons of a university in iCal
router.get("/:university/lessons.ics", async (req, res) => {
    const universita = universities.get(req.params.university);

    if (!universita) {
        res.status(404).send({error: 404, message: "Università non trovata"});
        return;
    }

    
    res.setHeader('Content-Type', 'text/calendar');
    res.setHeader('Content-Disposition', 'attachment; filename="lesson-timetable.ics"');
    const calendar = ical({ name: "Lesson Timetable"});


    try {
        const lessons = await universita.CacheLessons((req.query.course_id as string), (req.query.course_year as string));


        for (const lesson of lessons) {
            calendar.createEvent({
                start: lesson.starts_at,
                end: lesson.ends_at,
                summary: lesson.subject,
                description: lesson.Teacher,
                location: lesson.location
            });
        }

    res.send(calendar.toString());


    } catch (error: any) {
        
        const starts_at = new Date();
        starts_at.setHours(0, 0, 0, 0);
        const ends_at = new Date();
        ends_at.setHours(23, 59, 59, 999);

        if (error instanceof UniversityError && error.UserFault) {
            // res.status(400).send({error: 400, message: error.message});

            calendar.createEvent({
                start: starts_at,
                end: ends_at,
                summary: "Error 400",
                description: error.message,
            })
            res.send(calendar.toString());
            return;
        }

        console.error(`-------------------------------------------------`);
        console.error(`[TimeTable] Error => Where: ${req.url}`);
        console.error(`[TimeTable] Error => When: ${ParseDate(new Date)}`);
        console.error(`[TimeTable] Error => \`${error}\``);
        console.error(`-------------------------------------------------`);

  
        calendar.createEvent({
            start: starts_at,
            end: ends_at,
            summary: "Error 500",
            description: "Server error",
        })


        res.send(calendar.toString());
    }


});

//#endregion

export default router;