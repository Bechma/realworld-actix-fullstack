# Realworld with Actix + Tera + SQLx

You can check it online in https://realworld-fullstack.shuttleapp.rs/

## How to run it

1. Get a database running

```bash
docker run -e POSTGRES_PASSWORD=postgres -p 5432:5432 --name postgres postgres
```

2. Create a .env file from the example

```bash
cp .env.example .env
```

3. Run the application in offline mode(it will execute migrations and setup required postgres extension)

```bash
SQLX_OFFLINE=true cargo run
```

## How to test it

Make sure that you have the database setup from the previous steps and just run

```bash
cargo test
```

## How to deploy it

I'm using a brand new serverless approach for rust applications: [shuttle](https://www.shuttle.rs/). At this moment is in alpha but it looks promising:

1. Test the deploy locally:
```bash
cargo install cargo-shuttle
cargo shuttle login --api-key YOUR_API_KEY_HERE
SQLX_OFFLINE=true cargo shuttle run
```

2. Once everything is fine, the next step is to create the secrets:
```bash
# Remember to change the secret to a random and secure hex string
cp Secrets.toml.example Secrets.toml
```

3. Change the name of the project in `Shuttle.toml` and deploy it

```bash
# As the time of writting, they don't have a nice testing environment, so we can't test it during deployment
cargo shuttle deploy --no-test
```

## Extra notes

* I wanted to write 0 javascript, so the experience is like a MPA. Everything is written in Rust with Jinja2 style templates(Tera). Very fast, although the limitation of pure HTML is a major drawback if this is a real realworld application that needs to be maintained and extended.
* All features are applied. However, instead of following a JWT authentication, I used session based.
* In HTML, there's `<a>` to make get requests to other pages and `<form>` to make get or post requests. This limitation forced me to only make routes with GET and POST.
* The tests can be expanded more, however, during compilation time SQLx makes sure that all queries are well writen and returns what we expect(so that's somewhat tested =D).

TODO:
- [x] Implement commenting section
- [x] Buttons to Edit/Remove your articles
- [x] Fav/Unfav article
- [x] Follow/Unfollow users
- [x] "Your feed" works
- [ ] Improve error handling
- [x] Write some tests
- [ ] Polishing
- [x] Prepare deployment files
- [x] Deploy somewhere (shuttle)
- [x] Finish this TODO with the architecture notes and how to deploy
