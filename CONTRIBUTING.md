# contributing to multi-delegator

Thanks for helping improve this project. This document explains how to set up a dev environment, run checks, and open pull requests against [solana-program/multi-delegator](https://github.com/solana-program/multi-delegator).

## before you start

- Read the [README](README.md) for an overview, program id, and project layout.
- Architecture notes live under [docs/](docs/); start with [ADR-001](docs/001-multi-delegator-architecture.md) if you change on-chain behavior.
- The program is **not audited**. Treat changes that touch funds or authority as high risk and call them out clearly in your PR.

## fork and clone

Fork the repository on GitHub, then clone your fork and add the upstream remote:

```bash
git clone https://github.com/<your-username>/multi-delegator.git
cd multi-delegator
git remote add upstream https://github.com/solana-program/multi-delegator.git
```

Keep your `main` branch tracking `upstream/main`, and use a feature branch for changes:

```bash
git fetch upstream
git checkout -b your-branch-name upstream/main
```

## prerequisites

The [README prerequisites](README.md#prerequisites) list the toolchain: Rust, Solana CLI, pnpm, Just, and Surfpool (for integration tests and local flows).

Run `just setup` once after cloning to install Node dependencies and configure git hooks.

## build and test

Typical workflow:

| Task | Command |
|------|---------|
| Build program + idl + clients | `just build` |
| Program tests only | `just test-program` |
| Full suite (program + TS client integration tests) | `just test` |
| Format + lint (same as CI) | `just check` |

Fix formatting before pushing if `just check` fails: `just fmt` and `just lint` (see [justfile](justfile)).

## generated code

IDL and Codama-generated clients under `programs/multi_delegator/idl/`, `clients/typescript/src/generated/`, and `clients/rust/src/generated/` are produced by the build. If you change the on-chain program, run `just build` (or `just generate-idl` / `just generate-client` as appropriate) and commit the regenerated artifacts so CI stays green.

## pull requests

- Open PRs against `solana-program/multi-delegator` `main` from your fork.
- Describe what changed and why; link related issues if any.
- Ensure `just check` and `just test` pass locally when your change touches code they cover.

## community

- Use [GitHub issues](https://github.com/solana-program/multi-delegator/issues) for bugs and feature ideas.
- For security-sensitive reports, use the contact options the maintainers prefer (see repository README or security policy if present).
