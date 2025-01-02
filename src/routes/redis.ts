import { Router } from 'express';
import redis from '../redis';

const router = Router();

router.get("/purge", async (req, res) => {
    res.send("<p>Redis password:</p> <form action='/redis/purge' method='post' id='ciao'><input type='password' id='password' name='password'><input type='submit' value='submit'></form>");
});

router.post("/purge", async (req, res) => {

    //Get the user's honeypot status
    const ip = req.headers['x-forwarded-for'] || req.connection.remoteAddress;
    const honeypotUserKey = `honeypot:ip:${ip}`;
    const honeypotStatus = await redis.get(honeypotUserKey);

    // Block IP if it tried to access the endpoint more than 5 times
    if (honeypotStatus !== null && parseInt(honeypotStatus) > 5) {
        res.status(401).send({ code: 401, message: "Unauthorized" });
        return
    }

    // Check if the password is incorrect
    if (req.body.password !== process.env.REDIS_PASSWORD) {
        // Register the attempt
        if (honeypotStatus === null) {
            await redis.set(honeypotUserKey, 1, "EX", 60 * 60 * 2); // 2 hours
        } else {
            await redis.incr(honeypotUserKey);
            // Warn if the IP got blocked
            if (parseInt(honeypotStatus) > 4) {
                console.warn(`[Honeypot] IP ${ip} got blocked after 5 attempts`);
            }
        }
        res.status(401).send({ code: 401, message: "Unauthorized" });
    } 
    else
    {
        // Flush the redis cache
        await redis.flushall();
        res.send({ code: 200, message: "Cache purged" });
        console.log("[Redis] Cache purged");
    }

});

export default router;