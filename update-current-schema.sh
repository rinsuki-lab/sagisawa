#!/bin/bash
pg_dump -d "$DATABASE_URL" --schema-only --no-owner | grep -vE "^--" > current.sql