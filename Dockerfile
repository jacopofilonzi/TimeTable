FROM node:20-alpine AS builder

WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build


FROM node:20-alpine
COPY --from=builder /app/build /app/build
COPY --from=builder /app/public /app/public
COPY --from=builder /app/package*.json /app
COPY --from=builder /app/node_modules /app/node_modules
ENV PORT 3000
EXPOSE ${PORT}
CMD ["node", "/app/build/app.js"]