FROM node:22 AS builder

WORKDIR /app

# RUN npm set registry https://registry.npmjs.org
COPY package.json /app/package.json
COPY package-lock.json /app/package-lock.json
COPY .npmrc /app/.npmrc
RUN --mount=type=secret,id=ghapikey,required \
    export GH_API_KEY="$(cat /run/secrets/ghapikey)"; \ 
    echo "//npm.pkg.github.com/:_authToken=${GH_API_KEY}" >> ~/.npmrc
RUN npm ci

COPY .eslintrc.json .eslintignore .prettierrc .prettierignore tsconfig.json config.json /app/
COPY lib /app/lib
COPY migrations /app/migrations
COPY __tests__ /app/__tests__

RUN npm run build

COPY esbuild.config.js /app/esbuild.config.js
RUN npm run build:esbuild

COPY static /app/static

# --- Runtime image ---
FROM debian:buster-slim

WORKDIR /app

# Copy only the SEA binary and required assets
COPY --from=builder /usr/local/bin/node /app/node
COPY --from=builder /app/dist/* /app/dist/
COPY --from=builder /app/static /app/static
COPY --from=builder /app/es5/migrations /app/migrations
COPY --from=builder /app/config.json /app/config.json
# COPY --from=builder /launch.sh /launch.sh
# RUN chmod +x /launch.sh

USER nobody

CMD ["/app/node", "/app/dist/server.bundle.js"]