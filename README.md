# User-service

.env example
```
DATABASE_URL=postgres://postgres:user-service-data@localhost:5432/user-service-data
```

## How to run
Step 1. Run Cargo install
```bash
cargo install
```

Step 2. Run the docker-compose.yaml file in detached mode
```bash
docker-compose up -d
```

Step 3. Add the environment variables to the .env file
```bash
DATABASE_URL=postgres://postgres:user-service-data@localhost:5432/user-service-data
JWT_SECRET_KEY=super-secret-key
```

Step 4. Run the API
```bash
cargo run
```