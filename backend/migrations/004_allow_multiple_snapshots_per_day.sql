-- had to add this because the one-per-day constraint from 002 broke demo mode
-- where we wanted to show the chart updating in real-time
ALTER TABLE public.tvl_snapshots DROP CONSTRAINT IF EXISTS tvl_snapshots_snapshot_date_key;

-- adding ascending index too since chart queries sometimes need oldest-to-newest
CREATE INDEX IF NOT EXISTS idx_tvl_snapshots_time_asc ON public.tvl_snapshots(snapshot_time ASC);
