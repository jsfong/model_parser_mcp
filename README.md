# Introduction 
A mcp exposing service as per model parser use in https://github.com/jsfong/model_parser.

# Pre-requisite
Create a .env file. This file contain mandatory environment variables.
Configure
- `DATABASE_URL=postgres://postgres:admin@localhost/modelruntime` //Targeting DB that contained savedModel
- `CACHE_SIZE=5` //Cache size

# Docker-compose
Create a .env file and run the docker compose file as below:
```
version: '3'
services:
  model_query:
    image: "jsfong/model-parser-mcp:<refer to release version>"
    ports:
      - "8001:8001"
    volumes:
      - ./.env:/.env
```
