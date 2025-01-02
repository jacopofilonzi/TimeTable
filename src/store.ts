import * as fs from "fs";
import University from "./types/University.js";


// Load universities
export function getUniversities(): Map<string, University> {
    const universities = new Map<string, University>();
    const files = fs.readdirSync(__dirname + "/universities").filter(file => file.endsWith(".js"));
    for (const file of files) {
        const university = require(__dirname + "/universities/" + file).default;
        universities.set(university.Denomination.toLocaleLowerCase(), university);
    }
    return universities;
}



export const universities: Map<string, University> = getUniversities();

