import { Request } from "express";
import redis, { isRedisConnected } from "../redis.js";
import Course from "./Course.js";
import Lesson from "./Lesson.js";
import crypto from "crypto";

export type FNUniversityGetCourses = (query: Request["query"]) => Promise<Course[]>;
export type FNUniversityGetLessons = (query: Request["query"]) => Promise<Lesson[]>;

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
     * Get the denomination of the university
     * 
     * @returns {string} The denomination of the university
     */
    get Denomination(): string {
        return this.denomination;
    }
    //#endregion
    
    
    //#region Methods

    /**
     * Get the courses of the university
     * 
     * @param query express request query
     * @returns {Promise<Course[]>} The courses of the university
     */
    public async GetCourses(query: Request["query"]): Promise<Course[]> {

        //If redis is not connected
        if (!await isRedisConnected())
            return await this.getCourses(query);
        else
        {
            //Query redis for cached values
            const cache_courses = await redis.hget(`courses:${this.denomination.toLowerCase()}`, `courses`);

            //If cache is not empty return it
            if (cache_courses != null)
                return JSON.parse(cache_courses);


            //Cache is empty, fetch courses
            const fetch_courses = await this.getCourses(query);

            //Save on cache
            redis.hset(`courses:${this.denomination.toLowerCase()}`, `courses`, JSON.stringify(fetch_courses), "EX", 60 * 60 * 24 * 7 * 3); //3 weeks

            //Return fetched courses
            return fetch_courses;

        }

    }

    /**
     * Get the lessons of the university
     * 
     * @param query 
     * @returns {Promise<Lesson[]>} The lessons of the university
     */
    public async GetLessons(query: Request["query"]): Promise<Lesson[]> {
        //If redis is not connected
        if (!await isRedisConnected())
            return await this.getLessons(query);
        else
        {
            //Create a hash of the query for better storage
            const queryHash = crypto.createHash('sha256').update(JSON.stringify(query)).digest('hex');
        
            //Query redis for cached values
            const cache_lessons = await redis.hget(`lessons:${this.denomination.toLowerCase()}`, queryHash);

            //If cache is not empty return it
            if (cache_lessons != null)
                return JSON.parse(cache_lessons);

            //Cache is empty, fetch lessons
            const fetch_lessons = await this.getLessons(query);

            //Save on cache
            redis.hset(`lessons:${this.denomination.toLowerCase()}`, queryHash, JSON.stringify(fetch_lessons), "EX", 60 * 60 * 24 * 3); //3 days

            //Return fetched lessons
            return fetch_lessons;
        }

    }

    //#endregion
}
