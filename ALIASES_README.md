# Cube.js Development Aliases

Comprehensive bash/zsh aliases for developing Cube.js from the current directory, with special support for Docker containers and build commands.

## Files Included

| File | Purpose |
|------|---------|
| `.cube-aliases` | Main aliases definition file (source this in your shell) |
| `setup-aliases.sh` | Automated setup script (recommended) |
| `CUBE_ALIASES_SETUP.md` | Detailed setup and usage guide |

## Quick Start (30 seconds)

### Method 1: Automatic Setup (Recommended)

```bash
cd /path/to/cube
chmod +x setup-aliases.sh
./setup-aliases.sh
source ~/.bashrc  # or ~/.zshrc for zsh
cube-help
```

### Method 2: Manual Setup

```bash
# Add this to ~/.bashrc or ~/.zshrc
if [ -f /path/to/cube/.cube-aliases ]; then
  source /path/to/cube/.cube-aliases
fi
```

Then reload:
```bash
source ~/.bashrc  # or ~/.zshrc
cube-help
```

### Method 3: Temporary (Current Session)

```bash
source /path/to/cube/.cube-aliases
cube-help
```

---

## What You Get

### 50+ Aliases for:

- ✅ **Installation**: `cube-install`, `cube-setup`
- ✅ **Building**: `cube-build`, `cube-watch`, `cube-clean`, `cube-rebuild`
- ✅ **Development**: `cube-dev`, `cube-dev-debug`, `cube-dev-trace`
- ✅ **Testing**: `cube-test`, `cube-test-watch`, `cube-test-coverage`
- ✅ **Linting**: `cube-lint`, `cube-lint-fix`
- ✅ **CubeSQL/Rust**: `cube-sql-build`, `cube-sql-test`, `cube-sql-run`
- ✅ **Docker**: `cube-docker-up`, `cube-docker-down`, `cube-docker-exec`
- ✅ **Database**: `cube-postgres-start`, `cube-postgres-connect`
- ✅ **Utilities**: `cube-help`, `cube-info`, `cube-new-project`

---

## Usage Examples

### First Time Setup

```bash
# Install everything
cube-setup

# Or step by step
cube-install
cube-native-build
cube-link-native
```

### Daily Development

```bash
# Terminal 1: Development server
cube-dev

# Terminal 2: Watch for changes
cube-watch

# Terminal 3: Run tests
cube-test-watch
```

### CubeSQL Development

```bash
# Terminal 1: Start Cube.js
cube-dev

# Terminal 2: Set env vars and run CubeSQL
export CUBESQL_TESTING_CUBE_URL="http://localhost:4000/cubejs-api"
export CUBESQL_TESTING_CUBE_TOKEN="test-token"
cube-sql-run

# Terminal 3: Run e2e tests
cube-sql-test

# Run specific test
cube-sql-test-name cancel_simple_query
```

### Docker Development

```bash
# Build image
cube-docker-build

# Start stack
cube-docker-up-d

# Run commands in container
cube-docker-yarn install
cube-docker-npm run build
cube-docker-node --version

# Connect to shell
cube-docker-shell

# View logs
cube-docker-logs
```

### Database Testing

```bash
# Start PostgreSQL
cube-postgres-start

# Create test project
cube-new-project my-test postgres

# Connect to database
cube-postgres-connect

# Stop when done
cube-postgres-stop
```

---

## Common Workflows

### Quick Development Iteration

```bash
cube-build && cube-dev
```

### Full Quality Check

```bash
cube-build && cube-lint && cube-test
```

### CubeSQL E2E Testing

```bash
# Terminal 1
cube-dev

# Terminal 2
export CUBESQL_TESTING_CUBE_URL="http://localhost:4000/cubejs-api"
export CUBESQL_TESTING_CUBE_TOKEN="test-token"
cube-sql-test
```

### Docker Development Cycle

```bash
cube-docker-build
cube-docker-up-d
cube-docker-logs
cube-docker-exec yarn install
cube-docker-down
```

---

## All Available Aliases

```
SETUP & INITIALIZATION
  cube-install              Install all dependencies
  cube-setup                Full setup (install + build native + link)
  cube-new-project NAME     Create a new test project

BUILD COMMANDS
  cube-build                Build all TypeScript packages
  cube-watch                Watch mode for TypeScript
  cube-clean                Clean build artifacts
  cube-rebuild              Clean and rebuild
  cube-native-build         Build native components
  cube-native-build-python  Build native with Python support
  cube-link-native          Link native module globally

LOCAL DEVELOPMENT
  cube-dev                  Start development server
  cube-dev-debug            Start with debug logging
  cube-dev-trace            Start with trace logging
  cube-dev-port PORT        Start on specific port

TESTING
  cube-test                 Run all tests
  cube-test-watch           Run tests in watch mode
  cube-test-coverage        Run tests with coverage
  cube-test-package NAME    Test specific package

LINTING & CODE QUALITY
  cube-lint                 Run all linters
  cube-lint-fix             Fix linting issues
  cube-lint-npm             Lint package.json files

CUBESQL / RUST
  cube-sql-build            Build CubeSQL
  cube-sql-release          Build CubeSQL release
  cube-sql-test             Run CubeSQL e2e tests
  cube-sql-test-name NAME   Run specific test
  cube-sql-snapshots        Review CubeSQL snapshots
  cube-sql-unit             Run CubeSQL unit tests
  cube-sql-fmt              Format Rust code
  cube-sql-lint             Lint Rust code
  cube-sql-run              Run CubeSQL server

DOCKER
  cube-docker-build         Build Docker image
  cube-docker-up            Start Docker Compose
  cube-docker-up-d          Start Docker Compose (detached)
  cube-docker-down          Stop Docker Compose
  cube-docker-logs          View Docker logs
  cube-docker-logs-all      View all Docker logs
  cube-docker-shell         Connect to Docker container
  cube-docker-exec CMD      Run command in Docker
  cube-docker-yarn CMD      Run yarn in Docker
  cube-docker-npm CMD       Run npm in Docker
  cube-docker-node CMD      Run node in Docker

DATABASE
  cube-postgres-start       Start PostgreSQL container
  cube-postgres-stop        Stop PostgreSQL container
  cube-postgres-connect     Connect to PostgreSQL

UTILITIES
  cube-help                 Show help message
  cube-info                 Show environment info
  cube-full-setup           Complete setup with tests
  cube-quick-dev            Quick build + dev server
  cube-ci-test              CI-like full test suite
```

---

## Docker Container Integration

These aliases make it easy to work with Docker containers:

```bash
# Run a single command in container
cube-docker-exec yarn install

# Run yarn command
cube-docker-yarn add package-name

# Run npm command
cube-docker-npm run build

# Run node directly
cube-docker-node --version

# Connect to shell
cube-docker-shell
```

---

## Features

### ✨ Smart Detection

- Automatically detects bash/zsh/fish shells
- Checks if Docker containers are running
- Validates environment variables for CubeSQL tests

### 🎨 Visual Feedback

- Colored output with blue, green, yellow, red indicators
- Progress messages for each step
- Success/error notifications

### 🐛 Error Handling

- Clear error messages with usage hints
- Validates preconditions (e.g., Docker running)
- Helpful troubleshooting guidance

### ⚡ Composable

- Build complex workflows by chaining aliases
- Supports pipes and redirects
- Works with all standard Unix tools

---

## Setup for Different Shells

### Bash

Add to `~/.bashrc`:
```bash
if [ -f /path/to/cube/.cube-aliases ]; then
  source /path/to/cube/.cube-aliases
fi
```

### Zsh

Add to `~/.zshrc`:
```bash
if [ -f /path/to/cube/.cube-aliases ]; then
  source /path/to/cube/.cube-aliases
fi
```

### Fish

Add to `~/.config/fish/config.fish`:
```fish
if test -f /path/to/cube/.cube-aliases
  source /path/to/cube/.cube-aliases
end
```

---

## Environment Variables

For CubeSQL e2e testing:

```bash
export CUBESQL_TESTING_CUBE_URL="http://localhost:4000/cubejs-api"
export CUBESQL_TESTING_CUBE_TOKEN="test-token"
```

Add to your shell config for persistence.

---

## Troubleshooting

### "command not found: cube-*"

Solution:
```bash
# Reload your shell config
source ~/.bashrc  # or ~/.zshrc

# Or manually source the aliases
source /path/to/cube/.cube-aliases

# Verify
cube-help
```

### "Cannot find module '@cubejs-backend/native'"

Solution:
```bash
cube-native-build
cube-link-native
```

### "Port 4000 already in use"

Solution:
```bash
# Use different port
cube-dev-port 4001

# Or kill existing process
lsof -i :4000
kill -9 <PID>
```

### Docker container not found

Solution:
```bash
# Start Docker Compose first
cube-docker-up-d

# Check it's running
docker ps | grep cube
```

---

## Files Modified by Setup

The setup script only modifies your shell configuration files:
- `~/.bashrc` (for bash)
- `~/.zshrc` (for zsh)
- `~/.config/fish/config.fish` (for fish)

It only adds a source command, nothing else is changed.

---

## Performance Tips

1. **Use watch mode** during development for faster feedback
2. **Run tests in parallel** with `&` and `wait`
3. **Use Docker** for isolated, reproducible environments
4. **Run specific tests** instead of the full suite when fixing issues

---

## Integration with IDEs

### VS Code

These aliases work great in VS Code's integrated terminal. Just make sure VS Code is using your shell (bash/zsh) and not sh.

### Other IDEs

Most IDEs with integrated terminals support bash/zsh, so the aliases should work automatically.

---

## Contributing & Extending

Want to add more aliases? Edit `.cube-aliases`:

```bash
# Add new alias
alias my-custom='_cube_echo_info "Running custom command" && your-command'

# Or create a function
my-function() {
  if [ -z "$1" ]; then
    _cube_echo_error "Usage: my-function <arg>"
    return 1
  fi
  _cube_echo_info "Running with $1"
  # Your command here
}
```

---

## Support

- **Setup Issues**: Run `./setup-aliases.sh` again
- **Missing Aliases**: Check `cube-help` for complete list
- **Docker Issues**: Run `docker ps` to verify containers
- **Build Issues**: Check `CUBE_JS_SETUP_GUIDE.md` for troubleshooting

---

## Version Info

- **Created**: 2025-11-27
- **Updated**: 2025-11-27
- **Aliases Count**: 50+
- **Supported Shells**: bash, zsh, fish

---

## Quick Reference Card

```bash
# Setup
./setup-aliases.sh                    # Automatic setup
cube-setup                            # Full setup
cube-new-project my-app               # Create test project

# Development
cube-dev                              # Start dev server
cube-watch                            # Watch TypeScript
cube-dev-debug                        # Debug mode

# Testing
cube-test                             # Run tests
cube-test-watch                       # Watch tests
cube-sql-test                         # CubeSQL tests

# Building
cube-build                            # Build TypeScript
cube-native-build                     # Build native
cube-lint-fix                         # Fix linting

# Docker
cube-docker-up                        # Start docker
cube-docker-down                      # Stop docker
cube-docker-shell                     # Connect to container

# Database
cube-postgres-start                   # Start PostgreSQL
cube-postgres-connect                 # Connect to DB

# Help
cube-help                             # Show all commands
cube-info                             # Show environment info
```

---

**Ready to develop! Run `cube-help` to get started.**
