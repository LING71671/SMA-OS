# Contributing Guide / 贡献指南

[中文](./CONTRIBUTING.md) | [English](./CONTRIBUTING_ZH.md)

---

Thank you for considering contributing to SMA-OS!

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Workflow](#development-workflow)
- [Code Standards](#code-standards)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

Please respect all contributors and maintain a friendly and constructive discussion atmosphere.

## How to Contribute

### Reporting Bugs

1. Search [Issues](https://github.com/LING71671/SMA-OS/issues) to check if the issue already exists
2. If not, create a new Issue including:
   - Clear title
   - Steps to reproduce
   - Expected behavior vs actual behavior
   - Environment information (OS, version, etc.)

### Submitting Feature Suggestions

1. Describe the feature you'd like to add in Issues
2. Explain the use case and value of the feature
3. Wait for maintainer feedback before starting implementation

### Submitting Code

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Make changes and add tests
4. Submit a Pull Request

## Development Workflow

### Environment Setup

```bash
# Clone repository
git clone https://github.com/LING71671/SMA-OS.git
cd SMA-OS

# Start infrastructure
docker-compose up -d

# Install dependencies
cd control-plane && cargo build
cd ../memory-bus && go mod download
cd ../orchestration && go mod download
cd ../observability-ui/web-dashboard && npm install
```

### Running Tests

```bash
# Go tests
cd memory-bus && go test -v ./...
cd ../orchestration && go test -v ./...

# Rust tests
cd control-plane && cargo test

# Frontend tests
cd observability-ui/web-dashboard && npm run lint
```

## Code Standards

### Rust

- Use `cargo fmt` to format code
- Use `cargo clippy` to check code quality
- All public APIs must have documentation comments (`///`)
- Use `Result<T, Error>` for error handling, don't use `unwrap()`

### Go

- Use `gofmt` to format code
- Use `golangci-lint` to check code quality
- Errors must be explicitly handled, don't ignore them
- Exported functions must have comments

### TypeScript

- Use `npm run lint` to check code
- Use TypeScript strict mode
- Use PascalCase naming for components

## Commit Messages

Commit message format:

```
<type>: <subject>

<body>

<footer>
```

### Type Categories

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation update
- `style`: Code formatting (no functionality change)
- `refactor`: Refactoring
- `test`: Test related
- `chore`: Build/tool related

### Example

```
feat: Add DAG execution timeout configuration

- Support global timeout setting
- Support per-task timeout configuration
- Auto-cleanup resources after timeout

Closes #123
```

## Pull Request Process

1. **Ensure tests pass**: All tests must pass before merging
2. **Update documentation**: If there are API changes, update relevant documentation
3. **One PR per feature**: Avoid mixing multiple unrelated changes in one PR
4. **Wait for review**: Maintainers will review your PR as soon as possible

### PR Checklist

- [ ] Code passes all tests
- [ ] New features have corresponding tests
- [ ] Documentation updated
- [ ] Commit message format is correct
- [ ] No merge conflicts

## Code Review

All submissions require code review. Review focuses on:

1. Code quality and readability
2. Test coverage
3. Documentation completeness
4. Compliance with project architecture

## Questions?

If you have any questions, you can:

1. Ask in [Issues](https://github.com/LING71671/SMA-OS/issues)
2. Check the [Deployment Documentation](../ops/DEPLOYMENT.md)

---

Thank you for your contribution!
