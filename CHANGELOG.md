# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

_Nothing yet_

## [0.1.0] - 2024-08-23

### Added

- Add a simple endpoint `/ingredient` to handle La Coctelera DB's ingredients.
- Add HTTP functions: `GET`, `POST`, `OPTIONS` to the endpoint `/ingredient`.
- Add OpenAPI docs to the API definitions and expose it using a service of the HTTP server.
  The service uses [Swagger UI](https://swagger.io/tools/swagger-ui/).
- Add tracing subsystem.
- Add a configuration subsystem to handle all the app's settings in a centralized and
  uniform way.
- Add CI to the repository.
- Add a deployment stage to publish the generated docs for the companion library of the app.
