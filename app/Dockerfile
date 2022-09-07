FROM node:18-alpine

COPY . /app
WORKDIR /app

ENV NODE_OPTIONS=--openssl-legacy-provider

RUN npm ci && \
    npm run build:docker && \
    rm -rf src/ node_modules/ && \
    npm set-script prepare "" && \
    npm ci --omit=dev && \
    npm i -g serve

ENV NODE_ENV=production

EXPOSE 3000

CMD serve -s build
