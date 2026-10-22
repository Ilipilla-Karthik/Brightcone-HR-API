# Brightcone HR Portal — Migration Inventory

## Overview
This document contains the complete inventory of existing functionality in the Brightcone HR Portal (Rust + Loco.rs backend) and maps each component to its replacement in the React + TypeScript + Tailwind CSS frontend.

---

## Migration Inventory Matrix

| Feature Name | Existing Page / Template | Existing Backend Endpoint | Request Format | Response Format | Auth Req. | Replacement React Component |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Public Landing Page** | `assets/views/landing.html` | `GET /` or `GET /welcome` | None | HTML (SSR Tera) | Public | `pages/LandingPage.tsx` (`features/auth/LandingPage.tsx`) |
| **Lamp Sign-In** | `assets/views/signin.html` | `POST /signin` | Form-encoded: `email`, `password` | 303 Redirect + Cookie `brightcone_user` | Public | `pages/SignInPage.tsx` with interactive Lamp cord physics & one-click demo fills |
| **Sign-Up (Register)** | `assets/views/signin.html` (tab) / `signup.html` | `POST /signup` | Form-encoded: `name`, `email`, `password`, `role` | 303 Redirect + Cookie `brightcone_user` | Public | `pages/SignUpPage.tsx` with role selection & validation |
| **Sign-Out** | `layout.html` (header) | `GET /signout` | None | 303 Redirect + clears cookie | Authenticated | Header / Profile Dropdown -> calls `POST /api/auth/logout` |
| **Session Restore / Me** | Header parsing in handlers | `get_authenticated_user` header check | Cookie: `brightcone_user` | User struct in Tera context | Optional | `api/auth.api.ts` -> `GET /api/auth/me` |
| **Live HR Dashboard** | `assets/views/dashboard.html` | `GET /dashboard` | Query: `msg`, `error` | HTML (SSR Tera) | Authenticated | `pages/DashboardPage.tsx` (`features/dashboard/Dashboard.tsx`) |
| **Dashboard Metrics** | Tera context `stats` | `AppState::get_stats()` | None | Tera context dict | Authenticated | `GET /api/dashboard/stats` |
| **Jobs Requisitions** | `assets/views/jobs.html` | `GET /jobs` | None | HTML (SSR Tera) | Authenticated | `features/recruitment/JobsTab.tsx` / `pages/RecruitmentPage.tsx` |
| **Publish Job** | `assets/views/jobs.html` | `POST /jobs/new` | Form: `title`, `department`, `location`, `skills`, `status` | 303 Redirect to `/jobs` | Authenticated | `POST /api/jobs` via `PostJobModal.tsx` |
| **Candidates Roster** | `assets/views/candidates.html` | `GET /candidates` | Query: `q`, `status` | HTML (SSR Tera) | Authenticated | `features/recruitment/PipelineTab.tsx` with Kanban & List views |
| **Register Candidate** | `assets/views/candidates.html` | `POST /candidates/new` | Form: `name`, `email`, `job_id`, `skills`, `status` | 303 Redirect to `/candidates` | Authenticated | `POST /api/candidates` via `AddCandidateModal.tsx` |
| **Update Candidate Stage** | `assets/views/candidates.html` | `POST /candidates/status` | Form: `candidate_id`, `status`, `redirect_to` | 303 Redirect | Authenticated | `PUT /api/candidates/:id/status` |
| **Interviews Schedule** | `assets/views/interviews.html` | `GET /interviews` | None | HTML (SSR Tera) | Authenticated | `features/recruitment/InterviewsTab.tsx` |
| **Schedule Interview** | `assets/views/interviews.html` | `POST /interviews/new` | Form: `candidate_id`, `interview_date`, `interviewer`, `interview_type`, `notes` | 303 Redirect | Authenticated | `POST /api/interviews` via `ScheduleInterviewModal.tsx` |
| **Employees Directory** | Demonstration records | *To be exposed via JSON REST API* | Query: `q`, `department`, `status` | JSON array of `Employee` | Authenticated | `features/employees/EmployeesPage.tsx` |
| **Add / Edit Employee** | Demonstration records | *To be exposed via JSON REST API* | JSON `EmployeePayload` | JSON `Employee` | Authenticated | `features/employees/EmployeeModal.tsx` |
| **CSV Import & Export** | UI specifications | *To be exposed via JSON REST API* | Multipart CSV / Download CSV | CSV data / confirmation | Authenticated | Export & Import buttons in `EmployeesPage.tsx` |
| **Performance Goals** | UI specifications | *To be exposed via JSON REST API* | Query / JSON `GoalPayload` | JSON array of `Goal` | Authenticated | `features/performance/GoalsView.tsx` |
| **Performance Reviews** | UI specifications | *To be exposed via JSON REST API* | Query / JSON `ReviewPayload` | JSON array of `Review` | Authenticated | `features/performance/ReviewsView.tsx` |
| **System Health Check** | `src/controllers/home.rs` | `GET /api` | None | JSON `{"app_name": "loco"}` | Public | Connected in `api/client.ts` for connectivity validation |
