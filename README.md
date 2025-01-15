<div align="center">
  <h1>La Coctelera (<i>back-end</i>)</h1>
  <p>
    <strong>
    La Coctelera is a collaborative data base in which cocktail recipes are collected and openly shared.
    </strong>
  </p>
  <p>

[![License](https://img.shields.io/github/license/felipet/lacoctelera_backend?style=flat-square)](https://github.com/felipet/lacoctelera_backend/blob/main/LICENSE)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/felipet/lacoctelera_backend/rust_main.yml?style=flat-square&label=CI%20status)
![Codecov](https://img.shields.io/codecov/c/github/felipet/lacoctelera_backend?token=82QNW2EJN1&style=flat-square)

  </p>
</div>

---

# What's La Coctelera?

La Coctelera is an open project to develop a collaborative data base of recipe cocktails. There are many websites that offer recipes for cocktails, aside from blogs, but there is no a public site in which people can post their recipes, vote them, and build a collaborative data base. This project aims to fill such gap.

The project is split in two developments: a back-end service (this repository); and a front-end service ([here][lacoctelera_frontend]). Our intention was to open a public REST API, so people could develop their own clients of the data base.

Willing to know more about this project? Then, go and check out the [project's web page](https://felipe.nubecita.eu/projects/lacoctelera/).

## La Coctelera's back-end service

An open REST API is offered to the community so they can implement custom clients of the data base, or simply connect easily with a *m2m* communication. The API is temporally hosted at [https://nubecita.eu/coctelera](https://nubecita.eu/coctelera/api/v0/).

The URL of the API is built with a fixed base (nubecita.eu/coctelera/api) and a reference to the target API version. This way the full API url would be *https://nubecita.eu/coctelera/api/{version}/*.

No stable releases of the API have been released, so only the **v0** is available right now.

The API is written in Rust and uses the [Actix-web](https://actix.rs/) framework. The documentation of the source code is found [here](https://felipet.github.io/lacoctelera_backend/lacoctelera/).

If you are willing to collaborate with the project, please drop an email to [torresfelipex1@gmail.com](mailto:torresfelipex1@gmail.com).

# Back-end development

>  Up to date information about the development can be found [here](https://felipe.nubecita.eu/categories/nubecita/).

All the content of the **main** branch is considered stable. Docker images are available of stable releases, and sometimes, of the main branch when important changes are merged into **main** but not ready for a stable relase.

If you need to run the back-end service locally, feel free to run it using Docker:

```bash
$ docker pull ghcr.io/felipet/lacoctelera_backend:main
$ docker run -d --network host --env-file lacoctelera.env ghcr.io/felipet/lacoctelera_backend:main
```
That way, the service will map to the host network and to the port specified in the configuration. Many configuration variables are needed in order to run the service (the data base connection, and so on). The easiest way to pass all the variables at once, is using an environment file and pass it to the image using `--env-file`. If you need more information about the runtime configuration variables, please check [this](https://felipet.github.io/lacoctelera_backend/lacoctelera/configuration/index.html) page of the documentation.

However, while developing, deploying the service via Docker is not the best idea, as it takes a lot of time to generate a new image, and huge amount of disk space. So if you plan to develop some feature for the back-end, better go to the next section to learn how to compile the binary and deploy it.

## Building the source code (hard way)

In order to build this service, the following dependences are needed:
- **Rust compiler and ***cargo*****. Installation guidelines found [here][rust-install].
- **MariaDB** Data Base Server. A simple script is included to launch a testing purpose DB using Docker, but feel free to replace that by a running instance that you may have.

## Build

To build the backend service for development, simply type:

```bash
$ cargo build
```
From the root of the project's directory. It will take some time depending on the power of your building machine. Source code from either the **main** branch or the **devel** branch shall build with no issues, so please, if you find any, don't hesitate to report it.

## Deploy

### Data Base Server

The application needs a DB server to store the Cocktail's information. Any MySQL data
base would work, but the scripts are ready for using [MariaDB](https://mariadb.com/).

For a development or testing deployment, the easiest way to go would be to launch a
DB instance using [Docker](https://www.docker.com/). If you don't have docker running
in your development computer, follow the official guidelines for installing it.

Once having Docker installed, simply run the script `scripts/init_db.bash` to launch
a new container using MariaDB's image. A set of migrations will be executed to prepare
the DB's schema. **However, no real data will be dump into the DB.** Thus before
attempting to retrieve data from the DB, push some into it!

The service expects several configuration variables to be set. Look at the _config_ directory to check what is needed.
Also, [this](https://felipet.github.io/lacoctelera_backend/lacoctelera/configuration/index.html) page explains
the purpose of each variable.

To run the service, simply type:

```bash
$ cargo run
```

And the back-end service will be available in the host address which is specified in the configuration files.

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
