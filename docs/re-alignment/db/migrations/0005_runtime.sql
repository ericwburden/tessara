-- runtime starter migration
CREATE TABLE form_responses (
  form_response_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  workflow_step_instance_id UUID NOT NULL,
  form_version_id UUID NOT NULL,
  status TEXT NOT NULL,
  answers_json JSONB NOT NULL,
  evaluated_state_json JSONB NOT NULL,
  submitted_at TIMESTAMPTZ NULL,
  updated_at TIMESTAMPTZ NOT NULL
);
