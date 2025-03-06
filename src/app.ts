import express, { Request, Response } from 'express';
import ParseDate from './utilities/parseDate';
import bodyParser = require('body-parser');
import * as dotenv from 'dotenv';
import path from 'path';
dotenv.config();

import AdminAuthManager from './utilities/adminAuth';
const adminAuthManager = new AdminAuthManager(process.env["ADMIN_OTP-TOKEN"]!);

const app = express();
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({     // to support URL-encoded bodies
    extended: true
})); 

// Middleware to filter and log every request
app.use((req: Request, res: Response, next) => {

    console.debug("[DEBUG]","new request", {
        url: req.url,
        method: req.method,
        ip_addr: req.ip,
        date: ParseDate(new Date())
    })

    next();
});
const port = process.env.PORT || 3000;


//#region Routes

//Add a static folder
app.use(express.static(path.join(__dirname, '..', 'public')));


//Register timetable routes
import timetable from './routes/timetable';
app.use('/timetable', timetable);

import redis from './routes/redis';
app.use('/redis', redis);
//#endregion


app.listen(port, () => {
    console.log(`[TimeTable] running on port ${port}`);
});



export {
    adminAuthManager
}
