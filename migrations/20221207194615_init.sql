CREATE TABLE IF NOT EXISTS Users (
    username text NOT NULL PRIMARY KEY,
    email text NOT NULL UNIQUE,
    password text NOT NULL,
    bio text NULL,
    image text NULL
);

CREATE TABLE IF NOT EXISTS Follows (
    follows text NOT NULL REFERENCES Users(username),
    influencer text NOT NULL REFERENCES Users(username),
    PRIMARY KEY (follows, influencer)
);

CREATE TABLE IF NOT EXISTS Articles (
    slug text NOT NULL PRIMARY KEY,
    author text NOT NULL REFERENCES Users(username),
    title text NOT NULL,
    description text NOT NULL,
    body text NOT NULL,
    created_at TIMESTAMPTZ NOT NULL default NOW(),
    updated_at TIMESTAMPTZ NOT NULL default NOW()
);

CREATE TABLE IF NOT EXISTS ArticleTags (
    article text NOT NULL REFERENCES Articles(slug),
    tag text NOT NULL,
    PRIMARY KEY (article, tag)
);

CREATE INDEX tags ON ArticleTags (tag);

CREATE TABLE IF NOT EXISTS FavArticles (
    article text NOT NULL REFERENCES Articles(slug),
    username text NOT NULL REFERENCES Users(username),
    PRIMARY KEY (article, username)
);

CREATE TABLE IF NOT EXISTS Comments (
    id int PRIMARY KEY,
    article text NOT NULL REFERENCES Articles(slug),
    username text NOT NULL,
    created_at TIMESTAMPTZ NOT NULL default NOW()
);
