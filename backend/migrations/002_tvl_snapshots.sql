-- Create snapshots table
CREATE TABLE IF NOT EXISTS public.tvl_snapshots (
    id SERIAL PRIMARY KEY,
    snapshot_date DATE NOT NULL DEFAULT CURRENT_DATE,
    snapshot_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    total_value_locked BIGINT NOT NULL,
    total_users INTEGER NOT NULL,
    active_vaults INTEGER NOT NULL,
    total_deposited BIGINT NOT NULL,
    total_withdrawn BIGINT NOT NULL,
    average_balance BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- originally wanted one snapshot per day for clean charts
    -- this gets removed in migration 004 for demo purposes
    UNIQUE(snapshot_date)
);

-- Create index for faster time-based queries
CREATE INDEX idx_tvl_snapshots_time ON public.tvl_snapshots(snapshot_time DESC);

-- seed with current state so charts don't start empty
INSERT INTO public.tvl_snapshots (
    total_value_locked,
    total_users,
    active_vaults,
    total_deposited,
    total_withdrawn,
    average_balance
)
SELECT 
    COALESCE(SUM(total_balance), 0)::BIGINT as total_value_locked,
    COUNT(DISTINCT owner) as total_users,
    -- only counting vaults that actually have funds
    COUNT(*) FILTER (WHERE total_balance > 0) as active_vaults,
    COALESCE(SUM(total_deposited), 0)::BIGINT as total_deposited,
    COALESCE(SUM(total_withdrawn), 0)::BIGINT as total_withdrawn,
    CASE 
        WHEN COUNT(DISTINCT owner) > 0 THEN (COALESCE(SUM(total_balance), 0) / COUNT(DISTINCT owner))::BIGINT
        ELSE 0
    END as average_balance
FROM public.vaults;

-- tried using this for the analytics endpoint but refresh was too slow
-- keeping it around in case we want aggregated daily views later
CREATE MATERIALIZED VIEW public.tvl_daily_summary AS
SELECT 
    snapshot_date as date,
    AVG(total_value_locked)::BIGINT as avg_tvl,
    MAX(total_value_locked) as max_tvl,
    MIN(total_value_locked) as min_tvl,
    AVG(total_users)::INTEGER as avg_users,
    MAX(total_users) as max_users
FROM public.tvl_snapshots
GROUP BY snapshot_date
ORDER BY snapshot_date DESC;

-- Create index on materialized view
CREATE INDEX idx_tvl_daily_summary_date ON public.tvl_daily_summary(date DESC);

-- Comment on table
COMMENT ON TABLE public.tvl_snapshots IS 'Historical TVL snapshots taken periodically for accurate charting and analytics';
