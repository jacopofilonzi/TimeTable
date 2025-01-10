import Redis from 'ioredis';
var REDIS_CONNECTION_ERROR_FIRED = false;

//---------------------------------------------------------------------------------------------------

const redis = new Redis({
    host: process.env.REDIS_HOST || "localhost",
    port: parseInt(process.env.REDIS_PORT || "6379"),
    password: process.env.REDIS_PASSWORD || undefined,
    db: parseInt(process.env.REDIS_DB || "0")
});

//---------------------------------------------------------------------------------------------------

redis.on("ready", () => {
    console.info("[Redis] Redis connection ready");
    REDIS_CONNECTION_ERROR_FIRED = false;
});

redis.on("error", (err) => {
    if (err.message.includes("getaddrinfo ENOTFOUND") || err.message.includes("connect ETIMEDOUT") || err.message.includes("connect ECONNREFUSED")) {
        if (!REDIS_CONNECTION_ERROR_FIRED) {
            console.warn("[Redis] WARNING: Redis server not found");
            REDIS_CONNECTION_ERROR_FIRED = true;
        }
    } else if (err.message.includes("WRONGPASS")) {
        if (!REDIS_CONNECTION_ERROR_FIRED) {
            console.error("[Redis] ERROR: Invalid Redis credentials");
            REDIS_CONNECTION_ERROR_FIRED = true;
        }
    } else {
        console.error("[Redis] ERROR:", err);
    }
});

//---------------------------------------------------------------------------------------------------

async function isRedisConnected() {
    if (redis.status !== 'ready') {
        return false;
    }
    
    try {
        await redis.ping();
        return true;
    } catch {
        return false;
    }
};

//---------------------------------------------------------------------------------------------------

export default redis;
export { isRedisConnected };