# Brightcone HR Portal — REST API Contracts

This document formalizes the HTTP REST API contracts implemented by the Rust + Loco.rs backend and consumed by the React + TypeScript frontend.

Base URL: `http://localhost:5150` (configurable via `VITE_API_BASE_URL`)

---

## 1. Authentication (`/api/auth`)

### 1.1 Login
- **Endpoint:** `POST /api/auth/login`
- **Request Body:**
  ```json
  {
    "email": "admin@brightcone.ai",
    "password": "admin123"
  }
  ```
- **Success Response (200 OK):**
  ```json
  {
    "user": {
      "id": 1,
      "name": "Recruitment Admin",
      "email": "admin@brightcone.ai",
      "role": "Senior Talent Lead",
      "created_at": "Oct 01, 2026"
    },
    "token": "admin@brightcone.ai"
  }
  ```
- **Error Response (401 Unauthorized):**
  ```json
  { "error": "Invalid email or password" }
  ```

### 1.2 Register
- **Endpoint:** `POST /api/auth/register`
- **Request Body:**
  ```json
  {
    "name": "Alex Mercer",
    "email": "alex@brightcone.ai",
    "password": "securepassword",
    "role": "Technical Recruiter"
  }
  ```
- **Success Response (201 Created):**
  ```json
  {
    "user": {
      "id": 5,
      "name": "Alex Mercer",
      "email": "alex@brightcone.ai",
      "role": "Technical Recruiter",
      "created_at": "Today"
    },
    "token": "alex@brightcone.ai"
  }
  ```

### 1.3 Logout
- **Endpoint:** `POST /api/auth/logout`
- **Headers:** `Cookie` or `Authorization: Bearer <token>`
- **Response (200 OK):**
  ```json
  { "message": "Signed out successfully" }
  ```

### 1.4 Current User (Me)
- **Endpoint:** `GET /api/auth/me`
- **Response (200 OK):**
  ```json
  {
    "user": {
      "id": 1,
      "name": "Recruitment Admin",
      "email": "admin@brightcone.ai",
      "role": "Senior Talent Lead",
      "created_at": "Oct 01, 2026"
    }
  }
  ```

---

## 2. Dashboard (`/api/dashboard`)

### 2.1 Dashboard Stats
- **Endpoint:** `GET /api/dashboard/stats`
- **Response (200 OK):**
  ```json
  {
    "total_jobs": 4,
    "total_candidates": 6,
    "shortlisted_candidates": 1,
    "scheduled_interviews": 3,
    "selected_candidates": 1,
    "total_employees": 8,
    "active_employees": 7,
    "employees_on_leave": 1,
    "goal_completion_rate": 78,
    "pending_reviews": 3
  }
  ```

---

## 3. Recruitment (`/api/recruitment` & `/api/jobs`, `/api/candidates`, `/api/interviews`)

### 3.1 Jobs
- `GET /api/jobs`: Returns array of `Job`
- `POST /api/jobs`:
  ```json
  {
    "title": "Principal AI Architect",
    "department": "Engineering",
    "location": "Hyderabad (Hybrid)",
    "skills": "PyTorch, Rust, LLM Agents",
    "status": "Active"
  }
  ```

### 3.2 Candidates
- `GET /api/candidates?q=...&status=...`: Returns array of `Candidate`
- `POST /api/candidates`:
  ```json
  {
    "name": "Jane Smith",
    "email": "jane@example.com",
    "job_id": 1,
    "skills": "Rust, Tokio, PostgreSQL",
    "status": "Applied"
  }
  ```
- `PUT /api/candidates/:id/status`:
  ```json
  {
    "status": "Shortlisted"
  }
  ```

### 3.3 Interviews
- `GET /api/interviews`: Returns array of `Interview`
- `POST /api/interviews`:
  ```json
  {
    "candidate_id": 1,
    "interview_date": "2026-10-20T14:30",
    "interviewer": "Sarah Jenkins",
    "interview_type": "System Design",
    "notes": "Discuss async runtime and distributed systems"
  }
  ```

---

## 4. Employees (`/api/employees`)

### 4.1 Demonstration Records Seeded
- `BC-1001` — Naresh Vemparala — Founder & CEO — Leadership — L9 — Hyderabad — no manager — Active
- `BC-1004` — Ananya Rao — People Ops Lead — People Ops — L6 — Hyderabad — Naresh Vemparala — Active
- `BC-1012` — Karthik Reddy — Principal AI Engineer — AI Research — L8 — Hyderabad — Naresh Vemparala — Active
- `BC-1019` — Priya Menon — Engineering Manager — Engineering — L7 — Bengaluru — Karthik Reddy — Active
- `BC-1023` — Daniel Brooks — Account Executive — Sales — L6 — San Francisco — Naresh Vemparala — Active
- `BC-1031` — Sneha Iyer — Product Designer — Design — L5 — Hyderabad — Priya Menon — On leave
- `BC-1038 — Rahul Varma — Backend Engineer — Engineering — L4 — Hyderabad — Priya Menon — Active
- `BC-1042` — Meera Joshi — Finance Manager — Finance — L6 — Hyderabad — Naresh Vemparala — Active

### 4.2 Endpoints
- `GET /api/employees?q=...&department=...&status=...`: Returns array of `Employee`
- `POST /api/employees`: Creates employee
- `PUT /api/employees/:id`: Updates employee
- `POST /api/employees/import`: Bulk import CSV
- `GET /api/employees/export`: CSV download stream

---

## 5. Performance (`/api/performance`)

### 5.1 Goals
- `GET /api/performance/goals`: Returns array of `Goal`
- `POST /api/performance/goals`: Creates new goal
- `PUT /api/performance/goals/:id/progress`: Updates goal completion percentage

### 5.2 Reviews
- `GET /api/performance/reviews`: Returns array of `PerformanceReview`
- `POST /api/performance/reviews`: Creates or updates performance review
