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

## [0.2.0] - 2024-09-23

- Full definition of the `/author`endpoint with a dummy implementation.
- Full definition of the `/recipe`endpoint with a dummy implementation.
- Definition of the `/echo` endpoint with a dummy implementation.
- Definition of the `/health` endpoint with a dummy implementation.
- Full definition of all the data objects needed to send/recipe data from the API to the clients.
- Introduced restricted access endpoints.

## [0.3.0] - 2024-10-28

- Add an authentication module to restrict the access to some endpoints of the API.
- Implement a process to request an API token using an human friendly web interface.
- Improved organisation of the HTML resources.
- Initial definition of HTML responses for the token request pages.
- Improved the CI scripts to reduce the completion time and distinguish between development and production jobs.

## [0.4.0] - 2024-11-08

- Full implementation of the `/author` endpoint.
- Integration of `actix_cors` to handle CORS in all the endpoints.
- Fixed bug with the client ID generation.

## [0.5.0] - 2024-11-14

### Added

- Improve the integration tests (GH-34).
- Improve the unit tests (GH-33).
- Increase test coverage to a minimum of 70% (GH-32).

### Bugs fixed

- `Author::shareable` should not be overwritten by `update_from` (GH-37).

### Bugs reported

- API docs for the method GET of `/author` are wrong for an empty search (GH-38).

## [0.6.0] - 2024-11-28

### Added

- Implemented GET & POST of the `/recipe` resource.

### Bugs fixed

- The return code of the GET methods of the `/recipe` resource match what is included in the docs.

## [0.7.0] - 2024-12-23

### Added

- Implemented GET of the `/ingredient` resource.
- Included configuration files and Github workflows to generate a Docker image of the service.

## [0.8.0] - 2025-01-16

### Added

- Modified the style sheet of the API docs page.
- Improved the project's documentation.
- The service accepts a configuration parameter to change the deployment path of the resources.

## [0.9.0] - 2025-03-26


### Added

- Prepare the container image to be deployed by Podman.
- Improved metainfo of the container image.
- Prepared container image to be handled by systemd.
- Improved logs to be handled by systemd.
- Prepare the container image to support auto-updates using Podman and systemd.
