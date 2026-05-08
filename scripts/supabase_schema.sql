CREATE TABLE IF NOT EXISTS wf_patients (
    sync_id         UUID PRIMARY KEY,
    machine_id      TEXT,
    hn              TEXT NOT NULL,
    enrolled_at     TIMESTAMPTZ NOT NULL,
    enrolled_by     TEXT,
    status          TEXT NOT NULL DEFAULT 'active',
    indication      TEXT,
    target_inr_low  REAL NOT NULL DEFAULT 2.0,
    target_inr_high REAL NOT NULL DEFAULT 3.0,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL,
    deleted_at      TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS wf_visits (
    sync_id              UUID PRIMARY KEY,
    machine_id           TEXT,
    hn                   TEXT NOT NULL,
    visit_date           TIMESTAMPTZ NOT NULL,
    inr_value            REAL,
    inr_source           TEXT,
    current_dose_mgday   REAL,
    dose_detail          TEXT,
    new_dose_mgday       REAL,
    new_dose_detail      TEXT,
    new_dose_description TEXT,
    selected_dose_option TEXT,
    dose_changed         INTEGER NOT NULL DEFAULT 0,
    next_appointment     TIMESTAMPTZ,
    next_inr_due         TIMESTAMPTZ,
    physician            TEXT,
    notes                TEXT,
    side_effects         TEXT,
    adherence            TEXT,
    created_by           TEXT,
    reviewed_at          TIMESTAMPTZ,
    reviewed_by          TEXT,
    created_at           TIMESTAMPTZ NOT NULL,
    updated_at           TIMESTAMPTZ NOT NULL,
    deleted_at           TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS wf_dose_history (
    sync_id         UUID PRIMARY KEY,
    machine_id      TEXT,
    hn              TEXT NOT NULL,
    changed_at      TIMESTAMPTZ NOT NULL,
    old_dose_mgday  REAL,
    new_dose_mgday  REAL,
    old_detail      TEXT,
    new_detail      TEXT,
    reason          TEXT,
    inr_at_change   REAL,
    changed_by      TEXT,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL,
    deleted_at      TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS wf_appointments (
    sync_id              UUID PRIMARY KEY,
    machine_id           TEXT,
    hn                   TEXT NOT NULL,
    appt_date            TIMESTAMPTZ NOT NULL,
    appt_type            TEXT,
    status               TEXT NOT NULL DEFAULT 'scheduled',
    notes                TEXT,
    source_visit_id      INTEGER,
    generated_from_visit INTEGER NOT NULL DEFAULT 0,
    created_at           TIMESTAMPTZ NOT NULL,
    updated_at           TIMESTAMPTZ NOT NULL,
    deleted_at           TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS wf_outcomes (
    sync_id       UUID PRIMARY KEY,
    machine_id    TEXT,
    hn            TEXT NOT NULL,
    event_date    TIMESTAMPTZ NOT NULL,
    event_type    TEXT NOT NULL,
    description   TEXT,
    inr_at_event  REAL,
    action_taken  TEXT,
    created_by    TEXT,
    created_at    TIMESTAMPTZ NOT NULL,
    updated_at    TIMESTAMPTZ NOT NULL,
    deleted_at    TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS wf_patient_status_history (
    sync_id        UUID PRIMARY KEY,
    machine_id     TEXT,
    hn             TEXT NOT NULL,
    status         TEXT NOT NULL,
    reason         TEXT,
    effective_date TIMESTAMPTZ NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL,
    updated_at     TIMESTAMPTZ NOT NULL,
    deleted_at     TIMESTAMPTZ
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_wf_patients_hn ON wf_patients(hn);

CREATE INDEX IF NOT EXISTS idx_wf_patients_updated_at ON wf_patients(updated_at);
CREATE INDEX IF NOT EXISTS idx_wf_visits_updated_at ON wf_visits(updated_at);
CREATE INDEX IF NOT EXISTS idx_wf_dose_history_updated_at ON wf_dose_history(updated_at);
CREATE INDEX IF NOT EXISTS idx_wf_appointments_updated_at ON wf_appointments(updated_at);
CREATE INDEX IF NOT EXISTS idx_wf_outcomes_updated_at ON wf_outcomes(updated_at);
CREATE INDEX IF NOT EXISTS idx_wf_patient_status_history_updated_at ON wf_patient_status_history(updated_at);

CREATE INDEX IF NOT EXISTS idx_wf_visits_hn ON wf_visits(hn);
CREATE INDEX IF NOT EXISTS idx_wf_dose_history_hn ON wf_dose_history(hn);
CREATE INDEX IF NOT EXISTS idx_wf_appointments_hn ON wf_appointments(hn);
CREATE INDEX IF NOT EXISTS idx_wf_outcomes_hn ON wf_outcomes(hn);
CREATE INDEX IF NOT EXISTS idx_wf_patient_status_history_hn ON wf_patient_status_history(hn);

ALTER TABLE wf_patients ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_visits ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_dose_history ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_appointments ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_outcomes ENABLE ROW LEVEL SECURITY;
ALTER TABLE wf_patient_status_history ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS anon_all_wf_patients ON wf_patients;
DROP POLICY IF EXISTS anon_all_wf_visits ON wf_visits;
DROP POLICY IF EXISTS anon_all_wf_dose_history ON wf_dose_history;
DROP POLICY IF EXISTS anon_all_wf_appointments ON wf_appointments;
DROP POLICY IF EXISTS anon_all_wf_outcomes ON wf_outcomes;
DROP POLICY IF EXISTS anon_all_wf_patient_status_history ON wf_patient_status_history;
DROP POLICY IF EXISTS anon_own_machine_wf_patients_select ON wf_patients;
DROP POLICY IF EXISTS anon_own_machine_wf_patients_insert ON wf_patients;
DROP POLICY IF EXISTS anon_own_machine_wf_patients_update ON wf_patients;
DROP POLICY IF EXISTS anon_own_machine_wf_visits_select ON wf_visits;
DROP POLICY IF EXISTS anon_own_machine_wf_visits_insert ON wf_visits;
DROP POLICY IF EXISTS anon_own_machine_wf_visits_update ON wf_visits;
DROP POLICY IF EXISTS anon_own_machine_wf_dose_history_select ON wf_dose_history;
DROP POLICY IF EXISTS anon_own_machine_wf_dose_history_insert ON wf_dose_history;
DROP POLICY IF EXISTS anon_own_machine_wf_dose_history_update ON wf_dose_history;
DROP POLICY IF EXISTS anon_own_machine_wf_appointments_select ON wf_appointments;
DROP POLICY IF EXISTS anon_own_machine_wf_appointments_insert ON wf_appointments;
DROP POLICY IF EXISTS anon_own_machine_wf_appointments_update ON wf_appointments;
DROP POLICY IF EXISTS anon_own_machine_wf_outcomes_select ON wf_outcomes;
DROP POLICY IF EXISTS anon_own_machine_wf_outcomes_insert ON wf_outcomes;
DROP POLICY IF EXISTS anon_own_machine_wf_outcomes_update ON wf_outcomes;
DROP POLICY IF EXISTS anon_own_machine_wf_patient_status_history_select ON wf_patient_status_history;
DROP POLICY IF EXISTS anon_own_machine_wf_patient_status_history_insert ON wf_patient_status_history;
DROP POLICY IF EXISTS anon_own_machine_wf_patient_status_history_update ON wf_patient_status_history;

CREATE POLICY anon_all_wf_patients ON wf_patients FOR ALL TO anon USING (true) WITH CHECK (true);
CREATE POLICY anon_all_wf_visits ON wf_visits FOR ALL TO anon USING (true) WITH CHECK (true);
CREATE POLICY anon_all_wf_dose_history ON wf_dose_history FOR ALL TO anon USING (true) WITH CHECK (true);
CREATE POLICY anon_all_wf_appointments ON wf_appointments FOR ALL TO anon USING (true) WITH CHECK (true);
CREATE POLICY anon_all_wf_outcomes ON wf_outcomes FOR ALL TO anon USING (true) WITH CHECK (true);
CREATE POLICY anon_all_wf_patient_status_history ON wf_patient_status_history FOR ALL TO anon USING (true) WITH CHECK (true);