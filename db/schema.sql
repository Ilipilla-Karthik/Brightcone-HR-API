-- Brightcone HR API: Master Database Schema
-- Auto-updated with Time & Leave tables

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ==========================================
-- Time & Leave Schema
-- ==========================================

CREATE TABLE IF NOT EXISTS shifts (
    id BIGSERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    code VARCHAR(30) NOT NULL UNIQUE,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    break_duration_minutes INTEGER NOT NULL DEFAULT 60,
    grace_period_minutes INTEGER NOT NULL DEFAULT 15,
    is_overnight BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_shifts_break_duration CHECK (break_duration_minutes >= 0),
    CONSTRAINT chk_shifts_grace_period CHECK (grace_period_minutes >= 0),
    CONSTRAINT chk_shifts_times CHECK (
        (NOT is_overnight AND start_time < end_time) OR
        (is_overnight AND start_time > end_time)
    )
);

CREATE INDEX IF NOT EXISTS idx_shifts_code ON shifts(code);
CREATE INDEX IF NOT EXISTS idx_shifts_active ON shifts(is_active);

CREATE TABLE IF NOT EXISTS holidays (
    id BIGSERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    name VARCHAR(150) NOT NULL,
    holiday_date DATE NOT NULL,
    description TEXT,
    applicable_location VARCHAR(100),
    is_recurring BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_holidays_date_location
    ON holidays (holiday_date, applicable_location) NULLS NOT DISTINCT;

CREATE INDEX IF NOT EXISTS idx_holidays_date ON holidays(holiday_date);

CREATE TABLE IF NOT EXISTS leave_types (
    id BIGSERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    code VARCHAR(30) NOT NULL UNIQUE,
    description TEXT,
    is_paid BOOLEAN NOT NULL DEFAULT true,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_leave_types_code ON leave_types(code);
CREATE INDEX IF NOT EXISTS idx_leave_types_active ON leave_types(is_active);

CREATE TABLE IF NOT EXISTS leave_quotas (
    id BIGSERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    leave_type_id BIGINT NOT NULL,
    year INTEGER NOT NULL,
    entitled_days NUMERIC(4, 1) NOT NULL,
    applicable_group VARCHAR(100) NOT NULL DEFAULT 'ALL',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_leave_quotas_leave_type
        FOREIGN KEY (leave_type_id) REFERENCES leave_types(id)
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT chk_leave_quotas_days CHECK (entitled_days > 0),
    CONSTRAINT chk_leave_quotas_year CHECK (year >= 2000 AND year <= 2100),
    CONSTRAINT uq_leave_quotas_type_year_group UNIQUE (leave_type_id, year, applicable_group)
);

CREATE INDEX IF NOT EXISTS idx_leave_quotas_type ON leave_quotas(leave_type_id);
CREATE INDEX IF NOT EXISTS idx_leave_quotas_year_group ON leave_quotas(year, applicable_group);

-- ==========================================
-- Employees & Weekly Roster Schema
-- ==========================================

CREATE TABLE IF NOT EXISTS employees (
    id BIGSERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    name VARCHAR(150) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    role VARCHAR(100) NOT NULL DEFAULT 'Employee',
    department VARCHAR(100) NOT NULL DEFAULT 'Engineering',
    default_shift_id BIGINT REFERENCES shifts(id) ON DELETE SET NULL,
    avatar_color VARCHAR(30) NOT NULL DEFAULT 'purple',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_employees_email ON employees(email);
CREATE INDEX IF NOT EXISTS idx_employees_active ON employees(is_active);

CREATE TABLE IF NOT EXISTS roster_assignments (
    id BIGSERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    employee_id BIGINT NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    roster_date DATE NOT NULL,
    shift_id BIGINT REFERENCES shifts(id) ON DELETE SET NULL,
    is_off BOOLEAN NOT NULL DEFAULT false,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_roster_employee_date UNIQUE (employee_id, roster_date)
);

CREATE INDEX IF NOT EXISTS idx_roster_date ON roster_assignments(roster_date);
CREATE INDEX IF NOT EXISTS idx_roster_emp_date ON roster_assignments(employee_id, roster_date);

CREATE TABLE IF NOT EXISTS roster_publications (
    id BIGSERIAL PRIMARY KEY,
    week_start_date DATE NOT NULL UNIQUE,
    published_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    published_by VARCHAR(100) NOT NULL DEFAULT 'Ananya Rao',
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_roster_pub_week ON roster_publications(week_start_date);

CREATE TABLE IF NOT EXISTS swap_requests (
    id BIGSERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    requester_id BIGINT NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    target_employee_id BIGINT REFERENCES employees(id) ON DELETE CASCADE,
    request_date DATE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_swap_requests_status ON swap_requests(status);

-- ==========================================
-- Daily Attendance Schema
-- ==========================================

CREATE TABLE IF NOT EXISTS daily_attendance (
    id BIGSERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    employee_id BIGINT NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    attendance_date DATE NOT NULL,
    attendance_status VARCHAR(50) NOT NULL DEFAULT 'PRESENT',
    in_time TIME,
    out_time TIME,
    hours_worked NUMERIC(5, 2) NOT NULL DEFAULT 0.0,
    entry_status VARCHAR(50) NOT NULL DEFAULT 'RECORDED',
    is_archived BOOLEAN NOT NULL DEFAULT false,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uq_daily_attendance_emp_date UNIQUE (employee_id, attendance_date),
    CONSTRAINT chk_attendance_status CHECK (attendance_status IN ('PRESENT', 'ABSENT', 'HALF_DAY', 'ON_LEAVE')),
    CONSTRAINT chk_entry_status CHECK (entry_status IN ('RECORDED', 'PENDING', 'REGULARISED')),
    CONSTRAINT chk_hours_worked CHECK (hours_worked >= 0.0 AND hours_worked <= 24.0)
);

CREATE INDEX IF NOT EXISTS idx_daily_attendance_emp ON daily_attendance(employee_id);
CREATE INDEX IF NOT EXISTS idx_daily_attendance_date ON daily_attendance(attendance_date);
CREATE INDEX IF NOT EXISTS idx_daily_attendance_status ON daily_attendance(attendance_status);
CREATE INDEX IF NOT EXISTS idx_daily_attendance_entry_status ON daily_attendance(entry_status);
CREATE INDEX IF NOT EXISTS idx_daily_attendance_archived ON daily_attendance(is_archived);

-- ==========================================
-- Attendance Correction Requests Schema
-- ==========================================

CREATE TABLE IF NOT EXISTS attendance_correction_requests (
    id BIGSERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    employee_id BIGINT NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    attendance_date DATE NOT NULL,
    correction_type VARCHAR(50) NOT NULL,
    requested_time TIME NOT NULL,
    reason TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    previous_in_time TIME,
    previous_out_time TIME,
    previous_hours_worked NUMERIC(5, 2),
    previous_status VARCHAR(50),
    reviewer_name VARCHAR(150),
    rejection_reason TEXT,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chk_correction_type CHECK (correction_type IN ('MISSED_CLOCK_IN', 'MISSED_CLOCK_OUT')),
    CONSTRAINT chk_correction_status CHECK (status IN ('PENDING', 'APPROVED', 'REJECTED'))
);

CREATE INDEX IF NOT EXISTS idx_corr_req_emp ON attendance_correction_requests(employee_id);
CREATE INDEX IF NOT EXISTS idx_corr_req_date ON attendance_correction_requests(attendance_date);
CREATE INDEX IF NOT EXISTS idx_corr_req_status ON attendance_correction_requests(status);
CREATE UNIQUE INDEX IF NOT EXISTS uq_pending_correction_request
ON attendance_correction_requests
    (employee_id, attendance_date, correction_type)
WHERE status = 'PENDING';



