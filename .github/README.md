<h1 align="center">
  <img src="/.github/assets/diem_logo.svg">
</h1>

<p align="center">
  <i align="center">Diem Is an Environment Manager</i> for students of 42
</p>

<h4 align="center">
  <a href="https://profile.intra.42.fr/users/elagouch"><img alt="School 42 badge" src="https://img.shields.io/badge/-elagouch-020617?labelColor=020617&color=5a45fe&logo=42"></a>
  <img alt="MIT license" src="https://img.shields.io/badge/License-MIT-ef00c7?logo=creativecommons&logoColor=fff&labelColor=020617">
  <img alt="Made in Rust" src="https://img.shields.io/badge/Made_in-Rust-ff2b89?logo=rust&logoColor=fff&labelColor=020617">
  <img alt="Package version" src="https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fairone01%2Fdiem%2Frefs%2Fheads%2Fmain%2FCargo.toml&query=%24.workspace.package.version&&logo=rust&logoColor=fff&label=Version&labelColor=020617&color=ff8059">
  <img alt="GitHub contributors" src="https://img.shields.io/github/contributors-anon/airone01/diem?logo=github&labelColor=020617&color=ffc248&label=Contributors">
  <img alt="GitHub last commit" src="https://img.shields.io/github/last-commit/airone01/diem?logo=github&labelColor=020617&color=f9f871&label=Last%20commit">
</h4>

## Features

- Package installation/uninstallation
- Management of package providers ("sources")

> [!NOTE]
> **Detailed package installation explaination**
> - Diem handles providers. A single provider could be called "John's repo" and be a GitHub repo of John.
> - John will define a configuration called an artifactory in his repo. Diem will look for that artifactory file and be taught the apps that this artifactory provides, as well as where the packages needed for these apps are located relative to the artifactory file.
> - When installing an app, diem will resolve the apps and the packages, and will take note of all the required packages and if packages are missing from the artifactory.
> - Then it will download/retrieve the packages from the artifactory and store them locally.
> - Finally, it will extract them to a directory defined in the configuration.

## Installation

TODO

## Usage

TODO

## Hosting/adding packages

TODO

## Roadmap

## 📋 Roadmap

| Category | Task | Priority | Status |
|----------|------|----------|--------|
| Feature | Basic structs | High | 🟢 Complete |
| Feature | Package installation | High | 🟡 Pending |

Legend:
- 🟢 Complete
- 🟡 In Progress/Partial
- 🔴 Not Started

## Contributing

TODO

<p align="center">
  <a href="https://en.wikipedia.org/wiki/Carpe_diem"><i align="center"><sub>Carpe Diem 🤘</sub></i></a>
</p>
