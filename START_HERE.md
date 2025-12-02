# 🚀 START HERE - Cube.js Development Setup

Welcome! You have comprehensive Cube.js development tools and documentation. Start here to get oriented.

## ⚡ Quick Start (5 Minutes)

```bash
# 1. Install aliases
cd /path/to/cube
./setup-aliases.sh

# 2. Reload shell
source ~/.bashrc

# 3. Verify
cube-help
```

Done! You now have 50+ development commands available.

## 📚 What Was Created

You have **7 files** with ~84 KB of setup guides and 50+ bash aliases:

| File | Purpose | Read Time |
|------|---------|-----------|
| `.cube-aliases` | Main aliases file | - |
| `setup-aliases.sh` | Auto setup script | - |
| `ALIASES_INSTALLATION_GUIDE.md` | Getting started | 5 min |
| `ALIASES_README.md` | Quick reference | 10 min |
| `CUBE_ALIASES_SETUP.md` | Detailed guide | 15 min |
| `CUBE_JS_SETUP_GUIDE.md` | Complete reference | 20 min |
| `CREATED_FILES_SUMMARY.md` | File overview | 5 min |

## 🎯 Choose Your Path

### Path A: Super Quick (5 min) ⚡
1. Run `./setup-aliases.sh`
2. Run `cube-help`
3. Start using aliases

### Path B: Complete Setup (30 min) 🎯
1. Read `ALIASES_INSTALLATION_GUIDE.md`
2. Run `./setup-aliases.sh`
3. Read `ALIASES_README.md`
4. Try aliases from the guide

### Path C: Deep Dive (2 hours) 🏆
1. Read all documentation files
2. Run `./setup-aliases.sh`
3. Explore each alias
4. Create custom aliases

## ✨ What You Get

### 50+ Bash Aliases for:
- ✅ Setup & installation
- ✅ Building TypeScript
- ✅ Local development
- ✅ Testing & linting
- ✅ CubeSQL/Rust development
- ✅ Docker development
- ✅ Database management

### Example Commands:
```bash
cube-setup              # Full initial setup
cube-dev                # Start dev server
cube-test               # Run tests
cube-sql-test           # CubeSQL tests
cube-docker-up          # Start Docker
cube-postgres-start     # Start PostgreSQL
cube-help               # Show all commands
```

## 📖 Documentation Map

### Getting Started
- **Read First**: `ALIASES_INSTALLATION_GUIDE.md` (5 min)
- **Then Try**: `./setup-aliases.sh` (1 min)
- **Then Use**: `cube-help` command

### Command Reference
- **Daily Use**: `ALIASES_README.md` (10 min)
- **Advanced**: `CUBE_ALIASES_SETUP.md` (15 min)

### Complete Cube.js Setup
- **Full Guide**: `CUBE_JS_SETUP_GUIDE.md` (20 min)
  - Local development
  - PostgreSQL setup
  - Docker Compose
  - CubeSQL e2e testing

### About Files
- **Overview**: `CREATED_FILES_SUMMARY.md` (5 min)

## 🚀 Next Steps

1. **Install** (1 minute)
   ```bash
   ./setup-aliases.sh
   source ~/.bashrc
   ```

2. **Verify** (1 minute)
   ```bash
   cube-help
   cube-info
   ```

3. **Setup** (5 minutes)
   ```bash
   cube-setup
   ```

4. **Develop** (start coding!)
   ```bash
   cube-dev
   ```

## 💡 Common Workflows

### Daily Development
```bash
# Terminal 1
cube-watch &

# Terminal 2
cube-dev
```

### Testing
```bash
cube-test              # All tests
cube-sql-test          # CubeSQL tests
cube-test-watch        # Watch mode
```

### CubeSQL Development
```bash
cube-dev &             # Terminal 1

# Terminal 2
export CUBESQL_TESTING_CUBE_URL="http://localhost:4000/cubejs-api"
cube-sql-test
```

### Docker Development
```bash
cube-docker-up         # Start containers
cube-docker-shell      # Connect to container
cube-docker-logs       # View logs
```

## ❓ FAQ

**Q: Do I need to install manually?**
A: No! Run `./setup-aliases.sh` to auto-install.

**Q: Which shells are supported?**
A: bash, zsh, and fish. Auto-detected.

**Q: Can I customize the aliases?**
A: Yes! Edit `.cube-aliases` and add your own.

**Q: How do I remove aliases?**
A: Just don't source the file, or edit your shell config.

**Q: What if the setup fails?**
A: Check `ALIASES_INSTALLATION_GUIDE.md` troubleshooting section.

**Q: Where's the complete Cube.js guide?**
A: See `CUBE_JS_SETUP_GUIDE.md`.

## 🔐 Safety

All files are:
- ✅ Safe to install
- ✅ Won't modify code
- ✅ Won't modify Cube.js repository
- ✅ Easy to remove
- ✅ Well-documented

## 📊 Statistics

- **Aliases**: 50+
- **Functions**: 15+
- **Documentation**: ~70 KB
- **Setup Time**: < 1 minute
- **Shells Supported**: 3
- **Operating Systems**: 3

## 🎉 Ready?

```bash
# Install aliases
./setup-aliases.sh

# Reload shell
source ~/.bashrc

# Show all commands
cube-help

# Start developing!
cube-dev
```

---

## 📝 Files at a Glance

```
Cube Repository Root/
├── .cube-aliases                      (Main aliases - 50+ commands)
├── setup-aliases.sh                   (Auto setup - run this!)
├── ALIASES_INSTALLATION_GUIDE.md      (Quick start)
├── ALIASES_README.md                  (Command reference)
├── CUBE_ALIASES_SETUP.md              (Detailed guide)
├── CUBE_JS_SETUP_GUIDE.md             (Complete reference)
├── CREATED_FILES_SUMMARY.md           (File overview)
└── START_HERE.md                      (This file!)
```

## 🔗 Quick Links

| Purpose | File |
|---------|------|
| Get started | `ALIASES_INSTALLATION_GUIDE.md` |
| See all commands | `ALIASES_README.md` |
| Learn workflows | `CUBE_ALIASES_SETUP.md` |
| Full Cube.js setup | `CUBE_JS_SETUP_GUIDE.md` |
| File overview | `CREATED_FILES_SUMMARY.md` |
| Help in terminal | `cube-help` |

## 💬 Support

**In Terminal:**
```bash
cube-help      # Show all aliases
cube-info      # Environment info
```

**In Files:**
- `ALIASES_README.md` - Quick reference
- `ALIASES_INSTALLATION_GUIDE.md` - Getting started
- `CUBE_JS_SETUP_GUIDE.md` - Troubleshooting

---

## 🎯 Recommendation for First-Time Users

1. **Read**: `ALIASES_INSTALLATION_GUIDE.md` (5 min)
2. **Install**: `./setup-aliases.sh` (1 min)
3. **Try**: `cube-help` and `cube-info` (2 min)
4. **Use**: Pick a workflow and start coding!

---

**Happy developing! 🚀**

Next: Run `./setup-aliases.sh` and `cube-help`
