# Brightcone HR Portal — Frontend Migration Report

## 1. Overview
The **Brightcone HR Portal** has been migrated from server-side rendered Tera HTML templates to a **React 18 + TypeScript + Vite + Tailwind CSS** single-page application (SPA). The **Rust + Loco.rs + Axum + SeaORM/PostgreSQL** backend was strictly preserved as the authoritative REST API server and data persistence layer.

---

## 2. Architecture Comparison

| Layer | Previous Architecture | Migrated Target Architecture |
|---|---|---|
| **Frontend Framework** | Tera server-side templates | React 18, Vite 6, TypeScript 5.8 |
| **Routing** | Axum HTML redirect handlers | React Router 7 (SPA with Protected Routes) |
| **Styling** | Custom static CSS (`style.css`) | Tailwind CSS 3.4 + Sora Google Font typography |
| **Icons** | Custom SVGs / Lucide CDN | Lucide React |
| **HTTP Client** | Browser native `<form>` POST submissions | Centralized Axios Client (`frontend/src/api/client.ts`) |
| **Backend Framework** | Loco.rs 1.2 on Axum 0.8 & Tokio | **Preserved:** Loco.rs 1.2 on Axum 0.8 & Tokio |
| **API Architecture** | Mixed HTML template renders & JSON | Clean, typed JSON REST APIs under `/api` |
| **Database** | PostgreSQL + Persistent store fallback | **Preserved:** PostgreSQL 18 & `data/brightcone_db.json` |

---

## 3. Replaced Frontend Components

All server-rendered HTML templates were retired and replaced by modular, typed React components:

| Old Tera Template | Replaced By React Component | Feature Domain |
|---|---|---|
| `assets/views/landing.html` | `frontend/src/features/auth/LandingPage.tsx` | Landing page, hero, feature cards, CTA |
| `assets/views/signin.html` | `frontend/src/features/auth/SignInPage.tsx` + `LampCord.tsx` | Interactive physics lamp cord & authentication |
| `assets/views/signup.html` | `frontend/src/features/auth/SignUpPage.tsx` | User registration & validation |
| `assets/views/dashboard.html` | `frontend/src/features/dashboard/DashboardPage.tsx` | Live KPI stats, pipeline summary, upcoming interviews |
| Old employee UI / embedded | `frontend/src/features/employees/EmployeesPage.tsx` + `EmployeeModal.tsx` | Employee directory, CSV export/import, search & filters |
| `assets/views/jobs.html` | `frontend/src/features/recruitment/JobsTab.tsx` + `PostJobModal.tsx` | Job requisition cards & modal with INR referral bonus |
| `assets/views/candidates.html` | `frontend/src/features/recruitment/PipelineTab.tsx` + `CandidateModal.tsx` | Kanban candidate pipeline (Applied -> Screening -> Interview -> Offer -> Hired) |
| `assets/views/interviews.html` | `frontend/src/features/recruitment/InterviewsTab.tsx` + `InterviewModal.tsx` | Interview scheduling & calendar list |
| N/A (Embedded) | `frontend/src/features/recruitment/OffersTab.tsx` & `ReferralsTab.tsx` | Offer release & referral bonuses |
| N/A (New UI) | `frontend/src/features/performance/PerformancePage.tsx` + `GoalModal.tsx` + `ReviewModal.tsx` | Performance KPIs, Goals tracking, Reviews & Ratings |
| `assets/views/layout.html` | `frontend/src/components/layout/AppLayout.tsx`, `Sidebar.tsx`, `Topbar.tsx` | Enterprise sidebar, topbar, theme toggle, profile menu |

---

## 4. Preserved Backend Components

The Rust backend was protected and enhanced without rewriting domain logic:
1. **Core Runtime**: Loco.rs 1.2, Axum 0.8, Tokio asynchronous runtime.
2. **Database Engine**: SeaORM entities and raw PostgreSQL synchronization (`brightcone_recruitment` on port 5433 / `DATABASE_URL`) with fallback `data/brightcone_db.json`.
3. **Domain Models (`src/models/mod.rs`)**:
   - `User`, `Job`, `Candidate`, `Interview`, `Employee`, `Goal`, `PerformanceReview`, `DashboardStats`.
   - Business methods: `authenticate()`, `register_user()`, `add_job()`, `update_job_status()`, `add_candidate()`, `update_candidate_status()`, `schedule_interview()`, `add_employee()`, `update_employee()`, `add_goal()`, `update_goal_progress()`, `add_review()`.
4. **Data Seed & Persistence**:
   - Guaranteed persistence of the **8 required demonstration employees**:
     - `BC-1001` Naresh Vemparala (Founder & CEO, Leadership, L9, Hyderabad, Active)
     - `BC-1004` Ananya Rao (People Ops Lead, People Ops, L6, Hyderabad, Active)
     - `BC-1012` Karthik Reddy (Principal AI Engineer, AI Research, L8, Hyderabad, Active)
     - `BC-1019` Priya Menon (Engineering Manager, Engineering, L7, Bengaluru, Active)
     - `BC-1023` Daniel Brooks (Account Executive, Sales, L6, San Francisco, Active)
     - `BC-1031` Sneha Iyer (Product Designer, Design, L5, Hyderabad, On leave)
     - `BC-1038` Rahul Varma (Backend Engineer, Engineering, L4, Hyderabad, Active)
     - `BC-1042` Meera Joshi (Finance Manager, Finance, L6, Hyderabad, Active)
5. **CORS & Session Middleware**:
   - Configured in `config/development.yaml` to allow origin `http://localhost:5173` with credentials.
   - Dual authentication support: Bearer token header and HTTP-only cookie (`brightcone_user`).

---

## 5. API Endpoints Inventory

### Authentication
- `POST /api/auth/login`: Authenticates credentials, returns user profile + token, sets session cookie.
- `POST /api/auth/register`: Validates and registers new account.
- `POST /api/auth/logout`: Clears session cookie and invalidates user state.
- `GET /api/auth/me`: Retrieves current authenticated user session.

### Dashboard
- `GET /api/dashboard/stats`: Returns live metric KPIs (total employees, active employees, on leave, open jobs, candidates, goal completion rate, pending reviews).

### Employees
- `GET /api/employees`: Search, filter by department or status. Returns employee records.
- `POST /api/employees`: Adds new employee.
- `PUT /api/employees/{id}`: Updates existing employee.
- `GET /api/employees/export`: Exports employee database as downloadable CSV.
- `POST /api/employees/import`: Imports employee records from uploaded CSV.

### Recruitment
- `GET /api/jobs`: Lists open and active job requisitions.
- `POST /api/jobs`: Publishes a new job (supports referral bonus in INR).
- `PUT /api/jobs/{id}/status`: Updates job status (e.g. Active, Closed).
- `GET /api/candidates`: Lists candidates with stage & keyword filters.
- `POST /api/candidates`: Registers candidate and links to requisition.
- `PUT /api/candidates/{id}/status`: Transitions candidate stage (Applied -> Screening -> Interview -> Offer -> Hired).
- `GET /api/interviews`: Lists scheduled interviews.
- `POST /api/interviews`: Schedules interview with interviewer, date/time, and type.

### Performance
- `GET /api/performance/goals`: Lists employee OKRs and strategic goals.
- `POST /api/performance/goals`: Creates new goal.
- `PUT /api/performance/goals/{id}/progress`: Updates goal progress (0-100%).
- `GET /api/performance/reviews`: Lists performance evaluations, ratings, and feedback.
- `POST /api/performance/reviews`: Submits performance review.

### System & Health
- `GET /api/health`: Returns system health check (`{"status":"ok","app":"Brightcone HR Portal"}`).

---

## 6. Removed Obsolete Files
- `src/controllers/recruitment.rs` (replaced by typed JSON API controller `src/controllers/api.rs`).
- `assets/views/candidates.html`
- `assets/views/dashboard.html`
- `assets/views/interviews.html`
- `assets/views/jobs.html`
- `assets/views/landing.html`
- `assets/views/layout.html`
- `assets/views/signin.html`
- `assets/views/signup.html`
- `assets/static/css/style.css`

---

## 7. Verification & Tests

### Backend Tests
Executed via `cargo test`:
```
running 9 tests
test views::render::renders_home_view_with_i18n ... ok
test requests::api::test_api_auth_flow ... ok
test requests::api::test_api_dashboard_stats ... ok
test requests::page::can_get_home_page ... ok
test requests::api::test_api_performance_goals_and_reviews ... ok
test requests::api::test_api_employees_and_seed_data ... ok
test requests::api::test_api_recruitment_jobs_and_candidates ... ok
test requests::home::can_get_home ... ok
test requests::api::test_api_health ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Code Formatting & Linting
- `cargo fmt --check`: 0 diffs.
- `cargo clippy --all-targets -- -D warnings`: 0 warnings, passed.

### Frontend Compilation
- `npm run typecheck`: 0 TypeScript errors.
- `npm run build`: Production bundle built in `frontend/dist/` in ~3.4s.
