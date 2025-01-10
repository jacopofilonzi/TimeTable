# <div style="text-align: center;">TimeTable</div>
Add your university calendar to any ics-compatible calendar

<!-- ----------------------------------------------------------------------- -->

## <div style="text-align: center;">Setup docker deployment</div>
Use the following steps for a standard deployment:




1. <h4>Build the image:</h4>
```bash
$ docker build . -t timetable
```

2. <h4>Create a .env file and specify at least REDIS_PASSWORD</h4>


3. <h4>Deploy the container using provided docker-compose.yaml:</h4>
```bash
$ docker compose up -d
```


<!-- ----------------------------------------------------------------------- -->

## <div style="text-align: center;">Environment variables</div>

- **PORT** - [optional] <br>TimeTable app port. Leave empty to default to 3000.
- **REDIS_HOST** - [optional] <br>Redis hostname/IP. Leave empty if using the default docker-compose file.
- **REDIS_PORT** - [optional] <br>Redis host port. Leave empty to default to 6379
- **REDIS_PASSWORD** - REQUIRED <br>Redis password. <br>This password secures Redis and allows you to manually clear the cache via the `/redis/purge` endpoint.