import express from 'express';
import * as dotenv from 'dotenv';
import path from 'path';
import bodyParser = require('body-parser');
dotenv.config();

const app = express();
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({     // to support URL-encoded bodies
    extended: true
})); 
const port = process.env.PORT || 3000;


//#region Routes

//Add a static folder
app.use(express.static(path.join(__dirname, '..', 'public')));


//Register timetable routes
import timetable from './routes/timetable';
app.use('/timetable', timetable);

import redis from './routes/redis';
app.use('/redis', redis);

//Redirect old endpoint to new one
app.get("/orario/unicam/ics", async (req, res) => {
    //Redirect to new endpoint
    const queryParamsString = Object.keys(req.query).map(key => `${key}=${req.query[key]}`).join('&');
    res.redirect(301, `/timetable/unicam/lessons.ics?${queryParamsString}`);
});

//#endregion


app.listen(port, () => {
    console.log(`[TimeTable] running on port ${port}`);
});