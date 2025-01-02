import Redis from 'ioredis';


const redis = new Redis({
    host: process.env.REDIS_HOST || "localhost",
    port: parseInt(process.env.REDIS_PORT || "6379"),
    password: process.env.REDIS_PASSWORD || undefined,
    db: parseInt(process.env.REDIS_DB || "0")
});



redis.on("connect", () => {
    console.log("[Redis] Connected to Redis");
});

redis.on("reconnecting", () => {
    console.log("[Redis] Reconnecting to Redis");
});

redis.on("close", () => {
    console.log("[Redis] Redis connection closed");
});

redis.on("end", () => {
    console.log("[Redis] Redis connection ended");
});

redis.on("ready", () => {
    console.log("[Redis] Redis connection ready");
});

redis.on("error", (err) => {
    console.error("[Redis] ERROR:", err);
});

export default redis;