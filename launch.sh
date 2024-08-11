#!/bin/bash

echo "Updating DB ..."
/app/node_modules/.bin/knex --knexfile /app/es5/lib/knexfile.js --env main migrate:latest
RESULT=$?
if [ $RESULT -ne 0 ]; then
  echo "DB migration failed, aborting"
  exit 1;
fi

echo "Starting ..."
cd /app
exec "$@"
