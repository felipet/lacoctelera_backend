<div align="center">
  <h1>La Coctelera (<i>backend</i>)</h1>
  <p>
    <strong>
    La Coctelera is a simple web service in which cocktail recipes are collected and openly shared.
    </strong>
  </p>
  <p>

[![License](https://img.shields.io/github/license/felipet/lacoctelera_backend?style=flat-square)](https://github.com/felipet/lacoctelera_backend/blob/main/LICENSE)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/felipet/lacoctelera_backend/rust_main.yml?style=flat-square&label=CI%20status)
![Codecov](https://img.shields.io/codecov/c/github/felipet/lacoctelera_backend?token=82QNW2EJN1&style=flat-square)

  </p>
</div>

---

> [!Warning]
> This service is in an early stage of development.

This repository includes the backend side of La Coctelera web service. The frontend side of
the service is found [here][lacoctelera_frontend]. The service's backend is developed in Rust
using [Actix-Web][actix] as web framework.

# Build & Deploy

***Currently, only local deployment is supported.***

## Dependencies

In order to build this service, the following dependences are needed:
- **Rust compiler and ***cargo*****. Installation guidelines found [here][rust-install].
- **MariaDB** Data Base Server. A simple script is included to launch a testing purpose DB.

## Build

To build the backend service for development, simply type:

```bash
$ cargo build
```

From the root of the project's directory.

## Deploy

### Data Base Sever

The application needs a DB server to store the Cocktail's information. Any MySQL data
base would work, but the scripts are ready for using [MariaDB](https://mariadb.com/).

For a development or testing deployment, the easiest way to go would be to launch a
DB instance using [Docker](https://www.docker.com/). If you don't have docker running
in your development computer, follow the official guidelines for installing it.

Once having Docker installed, simply run the script `scripts/init_db.bash` to launch
a new container using MariaDB's image. A set of migrations will be executed to prepare
the DB's schema. **However, no real data will be dump into the DB.** Thus before
attempting to retrieve data from the DB, push some into it!

To run the service, simply type:

```bash
$ cargo run
```

# Development

Before making any commit to the repository, [pre-commit] shall be installed to check
that everything within the commit complies with the style rules of the repository.

Then, a ***git hook*** shall be installed. The hooks for this repository are located
at `.githooks`. These can be copied to `.git/hooks/` or used straight from such
location when telling ***git*** where to look for hooks:

```bash
$ git config core.hooksPath .githooks
```

A pre-push hook is also added to avoid pushing code that doesn't pass tests. If you
really aim to push code that doesn't pass tests for some reason, the following command
can be used:

```bash
$ git push --no-verify <remote> <branch>
```

[lacoctelera_frontend]: https://github.com/felipet/lacoctelera_frontend
[actix]: https://actix.rs/
[rust-install]: https://www.rust-lang.org/es/learn/get-started
[pre-commit]: https://pre-commit.com/#install
