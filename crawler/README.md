# Crawler

## Setup

1. Create the `secrets` directory.
```sh
mkdir secrets
```

2. Add your database password to the `secrets/db-password.txt` file.
```sh
echo 'SuperSecretPassword' > secrets/db-password.txt
```

3. Add your cache password to the `secrets/cache.env` file.
```sh
REDIS_PASSWORD=SuperSecretPassword
```

4. Set up the `.env` file.
```sh
DATABASE_URL=postgres://postgres:SuperSecretPassword@10.0.0.2/rse
CACHE_URL=redis://redis:SuperSecretPassword@10.0.0.3
```

