# Brightcone HR Portal — Setup & Running Guide

This guide details the exact steps and commands required to set up, develop, and test the **Brightcone HR Portal** (Rust Loco backend + React TypeScript Tailwind frontend).

---

## 1. Prerequisites

- **Operating System**: Windows (PowerShell)
- **Rust & Cargo**: Rust 1.80+ (tested on Rust 1.99 stable)
- **Node.js & npm**: Node.js v18+ (tested on v22.14.0, npm v10.9.2)
- **PostgreSQL**: PostgreSQL 14+ (optional: app automatically persists state in `data/brightcone_db.json` if PostgreSQL is unavailable)

---

## 2. Windows PowerShell Startup Commands

### Terminal 1: Rust Loco Backend
```powershell
# 1. Navigate to the root directory
cd "C:\bright_cone by reacttt - Copy"

# 2. Verify dependencies and compilation
cargo check

# 3. Start the Loco server
cargo run -- start
```
The backend starts and listens on **`http://localhost:5150`**.

### Terminal 2: React Frontend
```powershell
# 1. Navigate to the frontend directory
cd "C:\bright_cone by reacttt - Copy\frontend"

# 2. Install dependencies (only required on first setup)
npm.cmd install

# 3. Start the Vite development server
npm.cmd run dev
```
The React frontend starts and is accessible at **`http://localhost:5173`**.

---

## 3. Environment Variables Configuration

### Backend (`.env` in root)
```ini
# Server Configuration
LOCO_ENV=development
PORT=5150
BINDING=0.0.0.0

# Database Configuration (PostgreSQL / Supabase)
DATABASE_URL=postgres://postgres:Cherrybuddu@1526@127.0.0.1:5433/brightcone_recruitment
DB_HOST=127.0.0.1
DB_PORT=5433
DB_USER=postgres
DB_PASSWORD=Cherrybuddu@1526
DB_NAME=brightcone_recruitment
```

### Frontend (`frontend/.env`)
```ini
# Backend API Base URL
VITE_API_BASE_URL=http://localhost:5150
```

> **Security Note**: Never expose database passwords, PostgreSQL credentials, or server secret keys in `VITE_` frontend variables. The React SPA communicates solely over HTTP APIs to `http://localhost:5150`.

---

## 4. Default Demo Accounts

Use these pre-configured demonstration accounts to log in at `http://localhost:5173/signin`:

| Role | Email | Password |
|---|---|---|
| **Admin (Senior Talent Lead)** | `admin@brightcone.ai` | `admin123` |
| **HR Specialist / Recruiter** | `hr@brightcone.ai` | `password123` |
| **Technical Recruiter** | `maya@brightcone.ai` | `secure123` |
| **Hiring Manager** | `manager@brightcone.ai` | `password123` |

You can also click the quick-fill buttons (`Admin Demo` / `HR Demo`) on the sign-in card.

---

## 5. Running Verification & Tests

### Backend Tests (Cargo)
```powershell
cd "C:\bright_cone by reacttt - Copy"

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy --all-targets -- -D warnings

# Execute all unit and integration tests
cargo test
```

### Frontend Checks (TypeScript & Vite)
```powershell
cd "C:\bright_cone by reacttt - Copy\frontend"

# Typecheck with TypeScript strict mode
npm.cmd run typecheck

# Build optimized production bundle
npm.cmd run build
```

---

## 6. Troubleshooting

- **PowerShell execution policy block**: If `npm` gives an execution policy error on Windows, use `npm.cmd` instead.
- **Port 5150 already in use**: Check running processes with `Get-Process -Name brightcone-hr-portal* | Stop-Process` or kill background task.
- **CORS Errors**: The backend is configured in `config/development.yaml` with CORS allowing `http://localhost:5173`. If using a different frontend port, update `config/development.yaml`.
- **Database Connection**: If PostgreSQL is stopped, the backend automatically maintains persistent state in `data/brightcone_db.json`, ensuring zero disruption.
