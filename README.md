# Realworld with Actix + Tera + SQLx

You can check it online in https://realworld-actix-fullstack.onrender.com

## How to run it

You need a PostgreSQL database available locally or remotely.

```bash
docker run -e POSTGRES_PASSWORD=postgres -p 5432:5432 --name postgres postgres
cp .env.example .env
cargo run
```

The application reads these environment variables:

- `DATABASE_URL`
- `COOKIE_SECRET`
- `HOST`
- `PORT`

## How to test it

You will need a postgres database up and running locally in order to execute tests:

```bash
docker run -e POSTGRES_PASSWORD=postgres -p 5432:5432 --name postgres postgres
cargo test
```

## How to deploy it

1. Build and run the container locally:

```bash
docker build -t realworld-rust-fullstack .
docker run \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres \
  -e COOKIE_SECRET=your_long_random_secret \
  -e HOST=0.0.0.0 \
  -e PORT=8080 \
  realworld-rust-fullstack
```

2. For any container platform, configure these environment variables:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
COOKIE_SECRET=your_long_random_secret
HOST=0.0.0.0
PORT=8080
```

3. Then deploy the built image to your platform of choice.

The application runs SQL migrations automatically on startup.

## Extra notes

- I wanted to write 0 javascript, so the experience is like a MPA. Everything is written in Rust with Jinja2 style
  templates(Tera). Very fast, although the limitation of pure HTML is a major drawback if this is a real realworld
  application that needs to be maintained and extended.
- All features are applied. However, instead of following a JWT authentication, I used session based.
- In HTML, there's `<a>` to make get requests to other pages and `<form>` to make get or post requests. This limitation
  forced me to only make routes with GET and POST.
- The tests can be expanded more, however, during compilation time SQLx makes sure that all queries are well writen and
  returns what we expect(so that's somewhat tested =D).
