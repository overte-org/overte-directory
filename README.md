# Overte Directory

This is a WIP replacement for the overte-metavrese system, which will include functionality for federation, more advanced contact management and new features, i.e. calendars, groups, and more.

## Building

To build the server install the following tools:

- `cargo`, `rustc` (get MSRV from rust-toolchain.toml)
- `pnpm`

Install webcontent:
```sh
pnpm install
```

Build server:
```sh
cargo build
```

## Contributing

In order to contribute, fork the repository, develop a change in your repo, and create a pull request.

Try keeping changes self-contained and write an overview of your change in your PR description.

### LLM-Generated contributions

Contributions, issues and code reviews generated using Large Language Models are not accepted.

## ID schemas

username: `@<username>:<directory>`
canonical user id: `~<uuid>:<directory>`
worlds: `#<worldslug>:<directory>`
canonical world id: `!<uuid>:<directory>`
