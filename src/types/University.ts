import redis from "../redis.js";
import Course from "./Course.js";
import Lesson from "./Lesson.js";

export type FNUniversityGetCourses = (query: object) => Promise<Course[]>;
export type FNUniversityGetLessons = (query: object) => Promise<Lesson[]>;

export interface UniversityConstructorParams {
    name: string;
    denomination: string;
    getCourses: FNUniversityGetCourses;
    getLessons: FNUniversityGetLessons;
}

export default class University {
    //#region Properties
    private name: string;
    private denomination: string;
    private getCourses: FNUniversityGetCourses;
    private getLessons: FNUniversityGetLessons;
    //#endregion

    //#region Constructor
    constructor(options: UniversityConstructorParams) {
        this.name = options.name;
        this.denomination = options.denomination;
        this.getCourses = options.getCourses;
        this.getLessons = options.getLessons;
    }
    //#endregion

    //#region Getters/Setters
    /**
     * Get the name of the university
     * 
     * @returns {string} The name of the university 
     */    
    get Name(): string {
        return this.name;
    }

    /**
     * Set the name of the university
     * 
     * @param {string} name The name of the university
     */
    set Name(name: string) {
        this.name = name;
    }

    /**
     * Get the denomination of the university
     * 
     * @returns {string} The denomination of the university
     */
    get Denomination(): string {
        return this.denomination;
    }

    /**
     * Set the denomination of the university
     * 
     * @param {string} denomination The denomination of the university
     */
    set Denomination(denomination: string) {
        this.denomination = denomination;
    }

    /**
     * Get the courses of the university
     * 
     * @returns {FNUniversityGetCourses} The courses of the university
     */
    get GetCourses(): FNUniversityGetCourses {
        return this.getCourses;
    }

    /**
     * Set the courses of the university
     * 
     * @param {FNUniversityGetCourses} getCourses The courses of the university
     */
    set GetCourses(getCourses: FNUniversityGetCourses) {
        this.getCourses = getCourses;
    }

    /**
     * Get the lessons of the university
     * 
     * @returns {FNUniversityGetLessons} The lessons of the university
     */
    get GetLessons(): FNUniversityGetLessons {
        return this.getLessons;
    }

    /**
     * Set the lessons of the university
     * 
     * @param {FNUniversityGetLessons} getLessons The lessons of the university
     */
    set GetLessons(getLessons: FNUniversityGetLessons) {
        this.getLessons = getLessons;
    }
    //#endregion


    //#region Methods

    public async CacheCourses(): Promise<Course[]> {

        const cache_courses = await redis.hget(`courses:${this.denomination.toLowerCase()}`, `courses`);

        //If cache is not empty return it
        if (cache_courses != null)
            return JSON.parse(cache_courses);

        
        //Fech courses
        const fetch_courses = await this.GetCourses({});

        //Save on cache
        redis.hset(`courses:${this.denomination.toLowerCase()}`, `courses`, JSON.stringify(fetch_courses), "EX", 60 * 60 * 24 * 7 * 3); //3 weeks
        
        //Return fetched courses
        return fetch_courses;
    }

    public async CacheLessons(course_id: string, course_year: string): Promise<Lesson[]> {
    
        const cache_lessons = await redis.hget(`lessons:${this.denomination.toLowerCase()}:${course_id}`, `${course_year}`);

        //If cache is not empty return it
        if (cache_lessons != null)
        {
            return JSON.parse(cache_lessons);
        }

        //Fech lessons
        const fetch_lessons = await this.GetLessons({course_id, course_year});

        //Save on cache
        redis.hset(`lessons:${this.denomination.toLowerCase()}:${course_id}`, `${course_year}`, JSON.stringify(fetch_lessons), "EX", 60 * 60 * 24 * 3); //3 day

        //Return fetched lessons
        return fetch_lessons;


    
    }


    //#endregion
}
