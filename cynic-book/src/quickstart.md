# Quickstart

If you just want to get going with cynic and don't care too much about how it
works, this is the chapter for you.

#### Pre-requisites

There's a few things you'll need before you get started:

1. An existing rust project (though you can just run `cargo new` if you don't
   have one).
2. A GraphQL API that you'd like to query, and
   [a copy of its schema](https://github.com/prisma-labs/get-graphql-schema).
3. A GraphQL query that you'd like to run against the API. If you don't have
   one of these, you should probably use graphiql or graphql playground to get
   started, or you can use the one I provide below. For this quickstart I'll be
   assuming you're using a query without any arguments.

#### Setting up dependencies.

First things first: you need to add cynic to your dependencies. We'll also need
an HTTP client library. For the purposes of this quickstart we'll be using
surf but you can use any library you want. Open up your `Cargo.toml` and
add the following under the `[dependencies]` section:

```toml
cynic = { version = "2", features = ["http-surf"] }
surf = "2"
```

Note that we've added the `http-surf` feature flag of `cynic` - this pulls in
some `surf` integration code, which we'll be using. If you're using a different
HTTP client, you'll need a different feature flag or you may need to see the
[documentation for making an HTTP request manually][2].

You may also optionally want to install `insta` - a snapshot testing library
that can be useful for double checking your GraphQL queries are as expected.
Add this under `[dev-dependencies]` so it's available under test but not at
runtime:

```toml
[dev-dependencies]
insta = "1"
```

Run a `cargo check` to make sure this builds and you're good to go.

#### Adding your schema to the build.

You'll want to make sure the GraphQL schema for your build is available to your
builds. For example, you could put it at `src/schema.graphql` - the rest of
this tutorial will assume that's where you put the schema.

#### Building your query structs.

Cynic allows you to build queries from Rust structs - so you'll need to take
the query you're wanting to run and convert it into some rust structs. This can
be quite laborious and error prone for larger queries so cynic provides [`a
generator`][1] to help you get started.

Go to [https://generator.cynic-rs.dev][1] and select how you'd like to input
your schema. If the GraphQL API you wish to use is accessible on the internet
you can just link directly to it (although it will need to have CORS headers
enabled). Otherwise you can upload your schema to the generator.

Once you've provided the schema, you should be dropped into a GraphiQL
interface but with an extra panel that contains Rust generated from your query
& schema.

<!--
For example, I've chosen to add the star wars schema and the following query:

```graphql
TODO
```

and been given, the following rust code:

```rust
TODO
```

-->

#### Checking your query (optional)

Since cynic generates queries for you based on Rust structs, it's not always
obvious what the GraphQL queries look like. Sometimes you might want to run a
query manually via Graphiql, or you might just want to see what effects
changing the rust structs have on the query itself.

I find writing snapshot tests using `insta` useful for this purpose. Assuming
your query is called `AllFilmsQuery` like mine is, you can add the following to
the same file you put the struct output into:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_films_query_gql_output() {
        use cynic::QueryBuilder;

        let operation = AllFilmsQuery::build(());

        insta::assert_snapshot!(operation.query);
    }
}
```

You can now run this test with `cargo test`. It should fail the first time, as
you've not yet saved a snapshot. Run `cargo insta review` (you may need to
`cargo install insta` first) to approve the snapshot, and the test should succeed.

You can use this snapshot test to double check the query whenever you make
changes to the rust code, or just when you need some actual GraphQL to make a
query outside of cynic.

#### Making your query

Now, you're ready to make a query against a server. Cynic provides integrations
for the `surf` & `reqwest` HTTP clients. We'll use `surf` here, but it should
be similar for reqwest (or if you're using another HTTP library [see here][2]
for how to use cynic with it).

First, you'll want to build an `Operation` similar to how we did it in the
snapshot test above (again, swapping `AllFilmsQuery` for the name of your root
query struct):

```rust
use cynic::{QueryBuilder, http::SurfExt};

let operation = AllFilmsQuery::build(());
```

This builds an `Operation` struct with is serializable using
`serde::Serialize`. You should pass it in as the HTTP body using your HTTP
client and then make a request. For example, to use surf to talk to the
StarWars API (see the docs for `cynic::http` if you're using another client):

```rust
let response = surf::post("https://swapi-graphql.netlify.app/.netlify/functions/index")
    .run_graphql(&operation)
    .await
    .unwrap();
```

Now, assuming everything went well, you should have the response to your query
which you can do whatever you want with. And that's the end of the quickstart.

[1]: https://generator.cynic-rs.dev
[2]: ./manual-http-requests.md
