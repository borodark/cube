# Cube.js Aliases Setup Guide

Quick setup guide for using the Cube.js development aliases.

## Quick Installation

### Option 1: Temporary (Current Session Only)

```bash
# Source the aliases file
source /path/to/cube/.cube-aliases

# Or if you're in the cube directory
source ./.cube-aliases

# Verify it loaded
cube-help
```

### Option 2: Permanent (Bash)

Add to your `~/.bashrc`:

```bash
# Cube.js Development Aliases
if [ -f ~/path/to/cube/.cube-aliases ]; then
  source ~/path/to/cube/.cube-aliases
fi
```

Then reload:
```bash
source ~/.bashrc
```

### Option 3: Permanent (Zsh)

Add to your `~/.zshrc`:

```bash
# Cube.js Development Aliases
if [ -f ~/path/to/cube/.cube-aliases ]; then
  source ~/path/to/cube/.cube-aliases
fi
```

Then reload:
```bash
source ~/.zshrc
```

### Option 4: Permanent (Fish)

Add to your `~/.config/fish/config.fish`:

```fish
# Cube.js Development Aliases
if test -f ~/path/to/cube/.cube-aliases
  source ~/path/to/cube/.cube-aliases
end
```

Then reload:
```fish
source ~/.config/fish/config.fish
```

### Option 5: Global Symlink

Create a symlink in `/usr/local/etc/profile.d/` (macOS):

```bash
sudo ln -s /path/to/cube/.cube-aliases /usr/local/etc/profile.d/cube-aliases.sh
```

Or in `/etc/profile.d/` (Linux):

```bash
sudo ln -s /path/to/cube/.cube-aliases /etc/profile.d/cube-aliases.sh
```

---

## Verification

After installation, verify it's working:

```bash
# Show all available aliases
cube-help

# Check environment info
cube-info
```

---

## Common Workflows

### First-Time Setup

```bash
# One-command setup (installs, builds, links everything)
cube-setup

# Or step by step
cube-install
cube-native-build
cube-link-native
```

### Daily Development

```bash
# Terminal 1: Start development server
cube-dev

# Terminal 2: In another directory, start CubeSQL (with Cube.js running)
export CUBESQL_TESTING_CUBE_URL="http://localhost:4000/cubejs-api"
export CUBESQL_TESTING_CUBE_TOKEN="test-token"
cube-sql-run

# Terminal 3: Run e2e tests
cube-sql-test
```

### Testing Workflow

```bash
# Run all tests
cube-test

# Test in watch mode
cube-test-watch

# Test specific package
cube-test-package schema-compiler

# Run CubeSQL e2e tests
cube-sql-test

# Run specific test
cube-sql-test-name cancel_simple_query
```

### Building & Linting

```bash
# Build everything
cube-build

# Watch for changes
cube-watch

# Fix linting issues
cube-lint-fix

# Full quality check
cube-build && cube-lint && cube-test
```

### Docker Workflow

```bash
# Build Docker image
cube-docker-build

# Start full stack
cube-docker-up

# In another terminal, run commands in container
cube-docker-yarn install
cube-docker-npm run build

# Or connect to shell
cube-docker-shell

# View logs
cube-docker-logs
```

### Database Setup

```bash
# Start PostgreSQL container
cube-postgres-start

# Connect to it
cube-postgres-connect

# Stop it when done
cube-postgres-stop
```

### Create Test Project

```bash
# Create project with DuckDB (no external DB)
cube-new-project my-test-project

# Create project with PostgreSQL
cube-new-project my-pg-project postgres

# Start it
cd my-test-project
yarn link "@cubejs-backend/native"
yarn install
yarn dev
```

---

## Alias Reference

### Setup & Installation
| Alias | Description |
|-------|-------------|
| `cube-install` | Install all dependencies |
| `cube-setup` | Full setup (install + build native + link) |
| `cube-new-project NAME` | Create a new test project |

### Build Commands
| Alias | Description |
|-------|-------------|
| `cube-build` | Build all TypeScript packages |
| `cube-watch` | Watch mode for TypeScript |
| `cube-clean` | Clean build artifacts |
| `cube-rebuild` | Clean and rebuild |
| `cube-native-build` | Build native components |
| `cube-native-build-python` | Build native with Python support |
| `cube-link-native` | Link native module globally |

### Local Development
| Alias | Description |
|-------|-------------|
| `cube-dev` | Start development server |
| `cube-dev-debug` | Start with debug logging |
| `cube-dev-trace` | Start with trace logging |
| `cube-dev-port PORT` | Start on specific port |

### Testing
| Alias | Description |
|-------|-------------|
| `cube-test` | Run all tests |
| `cube-test-watch` | Run tests in watch mode |
| `cube-test-coverage` | Run tests with coverage |
| `cube-test-package NAME` | Test specific package |

### Linting & Code Quality
| Alias | Description |
|-------|-------------|
| `cube-lint` | Run all linters |
| `cube-lint-fix` | Fix linting issues |
| `cube-lint-npm` | Lint package.json files |

### CubeSQL / Rust
| Alias | Description |
|-------|-------------|
| `cube-sql-build` | Build CubeSQL |
| `cube-sql-release` | Build CubeSQL release |
| `cube-sql-test` | Run CubeSQL e2e tests |
| `cube-sql-test-name NAME` | Run specific test |
| `cube-sql-snapshots` | Review CubeSQL snapshots |
| `cube-sql-unit` | Run CubeSQL unit tests |
| `cube-sql-fmt` | Format Rust code |
| `cube-sql-lint` | Lint Rust code |
| `cube-sql-run` | Run CubeSQL server |

### Docker
| Alias | Description |
|-------|-------------|
| `cube-docker-build` | Build Docker image |
| `cube-docker-up` | Start Docker Compose |
| `cube-docker-up-d` | Start Docker Compose (detached) |
| `cube-docker-down` | Stop Docker Compose |
| `cube-docker-logs` | View Docker logs |
| `cube-docker-logs-all` | View all Docker logs |
| `cube-docker-shell` | Connect to Docker container |
| `cube-docker-exec CMD` | Run command in Docker |
| `cube-docker-yarn CMD` | Run yarn in Docker |
| `cube-docker-npm CMD` | Run npm in Docker |
| `cube-docker-node CMD` | Run node in Docker |

### Database
| Alias | Description |
|-------|-------------|
| `cube-postgres-start` | Start PostgreSQL container |
| `cube-postgres-stop` | Stop PostgreSQL container |
| `cube-postgres-connect` | Connect to PostgreSQL |

### Utilities
| Alias | Description |
|-------|-------------|
| `cube-help` | Show help message |
| `cube-info` | Show environment info |

### Composite Commands
| Alias | Description |
|-------|-------------|
| `cube-full-setup` | Complete setup with tests |
| `cube-quick-dev` | Quick build + dev server |
| `cube-ci-test` | CI-like full test suite |

---

## Environment Variables

For CubeSQL e2e testing, set these:

```bash
# Add to ~/.bashrc or ~/.zshrc
export CUBESQL_TESTING_CUBE_URL="http://localhost:4000/cubejs-api"
export CUBESQL_TESTING_CUBE_TOKEN="test-token"
```

Or set them before running tests:

```bash
export CUBESQL_TESTING_CUBE_URL="http://localhost:4000/cubejs-api"
export CUBESQL_TESTING_CUBE_TOKEN="test-token"
cube-sql-test
```

---

## Docker Container Commands

When using Docker Compose:

```bash
# Start stack
cube-docker-up-d

# Run commands in the container
cube-docker-yarn install
cube-docker-npm run build
cube-docker-node --version

# Or use cube-docker-exec for more flexibility
cube-docker-exec bash -c "yarn install && yarn build"

# Connect to shell
cube-docker-shell
```

---

## Troubleshooting

### Aliases Not Found

**Problem:** `command not found: cube-*`

**Solution:**
```bash
# Check if .cube-aliases was loaded
echo $BASH_VERSION  # If empty, you might be using sh instead of bash

# Manually source it
source /path/to/cube/.cube-aliases

# Or check the file path is correct
ls -la /path/to/cube/.cube-aliases
```

### Port Already in Use

**Problem:** `Error: listen EADDRINUSE: address already in use :::4000`

**Solution:**
```bash
# Use different port
cube-dev-port 4001

# Or kill the process
lsof -i :4000
kill -9 <PID>
```

### Native Module Issues

**Problem:** `Cannot find module '@cubejs-backend/native'`

**Solution:**
```bash
# Rebuild and relink native
cube-native-build
cube-link-native

# Or full setup
cube-setup
```

### Docker Container Not Running

**Problem:** `Docker container 'cube-server' is not running`

**Solution:**
```bash
# Start Docker Compose
cube-docker-up-d

# Check status
docker ps | grep cube

# View logs
cube-docker-logs
```

---

## Tips & Tricks

### Create an alias for your most-used workflow

```bash
# Add to ~/.bashrc
alias cube-dev-test='cube-dev & sleep 5 && cube-sql-test'
```

### Use function composition

```bash
# Chain commands
cube-build && cube-test && cube-dev

# Or use the composite commands
cube-quick-dev
cube-ci-test
```

### Customize log levels

```bash
# Very verbose logging
CUBEJS_LOG_LEVEL=trace cube-dev

# Or with native build
RUST_LOG=debug cube-sql-run
```

### Run tests in parallel

```bash
# Run multiple tests at once
cube-test-package schema-compiler & \
cube-test-package query-orchestrator & \
wait
```

---

## Advanced Usage

### Custom Command in Docker

```bash
# Run complex command
cube-docker-exec bash -c "
  cd /cube/conf && \
  yarn install && \
  yarn build && \
  yarn test
"
```

### Monitor Build

```bash
# Watch mode in one terminal
cube-watch

# While watching, run dev in another
cube-dev
```

### Full CI Simulation

```bash
# Run like CI would
cube-ci-test

# Or manually
cube-build && cube-lint && cube-test
```

---

## Integration with IDEs

### VS Code

Create `.vscode/settings.json`:

```json
{
  "terminal.integrated.shellArgs.linux": ["-i", "-l"],
  "terminal.integrated.env.linux": {
    "CUBESQL_TESTING_CUBE_URL": "http://localhost:4000/cubejs-api",
    "CUBESQL_TESTING_CUBE_TOKEN": "test-token"
  }
}
```

### Terminal Integration

Most modern terminals support shell integration, so just source `.cube-aliases` in your terminal startup file and you're good to go.

---

## Performance Tips

### Faster Builds

```bash
# Skip tests during build
cube-build

# vs
cube-ci-test
```

### Watch Mode Benefits

```bash
# Terminal 1: Watch for changes
cube-watch

# Terminal 2: Run dev
cube-dev

# Changes are compiled automatically!
```

### Cache Management

```bash
# Clean all cache
cube-clean

# Rebuild from scratch
cube-rebuild
```

---

## Support & Updates

If you find issues or want to add more aliases:

1. Edit `.cube-aliases` in the Cube repository
2. Test the changes
3. Consider submitting improvements to the project

Current version: 1.0.0
Last updated: 2025-11-27
